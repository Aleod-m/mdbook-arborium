use std::{collections::{BTreeMap, HashMap}, io::Write, ops::Range};

use anyhow::Result;
use arborium::theme::Theme;
use clap::{Arg, ArgMatches, Command, arg, command};
use mdbook_markdown::{MarkdownOptions, pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd, TextMergeWithOffset}};
use mdbook_preprocessor::{Preprocessor, PreprocessorContext, book::{Book, BookItem}, errors};
use semver::{Version, VersionReq};
use thiserror::Error;

const JS: &str = include_str!("./theme-selector.js");

struct SyntaxColor; 

impl SyntaxColor {
    pub fn new() -> SyntaxColor {
        SyntaxColor
    }
}

struct CodeBlockInfo {
    lang: String,
    code_span: Range<usize>,
    source_span: Range<usize>,
}

impl Preprocessor for SyntaxColor {
    fn name(&self) -> &str {
        "mdbook-arborium"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> errors::Result<Book> {
        let mut hi = arborium::Highlighter::new();
        book.for_each_mut(|item| {
            let BookItem::Chapter(chapter) = item else {return;};
            let parser = TextMergeWithOffset::new(mdbook_markdown::new_cmark_parser(&chapter.content, &MarkdownOptions::default()).into_offset_iter()).filter(|(e, _)| 
                matches!(
                    e, 
                    Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_))) 
                    | Event::End(TagEnd::CodeBlock) 
                    | Event::Text(_)
                )
            );

            let mut code_blocks: Vec<CodeBlockInfo> = Vec::new();
            let mut source: Option<(Range<usize>, String)> = None;

            for (event, range) in parser {

                match event {
                    Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(code))) => {
                        if arborium::get_language(&code).is_none() { 
                            continue;
                        };
                        source = Some((range, code.to_string()));
                    },
                    Event::End(TagEnd::CodeBlock) => {
                        source = None;
                    },
                    Event::Text(_) if (&source).is_some() => {
                        let (source_span, lang ) = source.take().unwrap();
                        code_blocks.push(CodeBlockInfo {
                            lang,
                            source_span,
                            code_span : range,
                        });
                    },
                    Event::Text(_) => {continue;}
                    _ => {
                        eprintln!("{:?} : {:?}", event, range);
                        unreachable!("Iterator is filtered !");
                    }
                }
            }
            let mut new_content = String::with_capacity(chapter.content.len() * 2);
            let mut cursor = 0;
            for code_info in code_blocks {
                let Ok(highlighted) = hi.highlight(&code_info.lang, &chapter.content[code_info.code_span]) else {
                    new_content.push_str(&chapter.content[cursor..code_info.source_span.end]);
                    cursor = code_info.source_span.end;
                    continue
                };

                new_content.push_str(&chapter.content[cursor..code_info.source_span.start]);
                new_content.push_str("<pre><code class=\"arborium\">");
                new_content.push_str(&highlighted);
                new_content.push_str("\n</code></pre>\n");
                cursor = code_info.source_span.end;
            }
            new_content.push_str(&chapter.content[cursor..]);
            chapter.content = new_content;
        });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> errors::Result<bool> {
        Ok(renderer == "html" || renderer == "markdown")

    }
}

#[derive(Debug, Error)]
enum Errors {
    #[error("Preprocessor failed")]
    Mdbook(#[source] errors::Error),

    #[error("Theme not found: ")]
    ThemeNotFound(String),
}

fn main() -> anyhow::Result<()> {
    let matches= command!()
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        ).subcommand(
            Command::new("install")
                        .subcommand(Command::new("list").about("List available themes"))
                        .subcommand(Command::new("all").about("Install all themes"))
                        .arg(arg!(theme: [THEME_NAME] "The selected theme to install"))
                        .about("Subcommand to install the necessary css.")
        ) .get_matches();


    let preproc = SyntaxColor::new();
    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&preproc, sub_args);
    }else if let Some(sub_arg) = matches.subcommand_matches("install") {
        let mut theme_map = arborium::theme::builtin::all()
            .into_iter()
            .map(|t| (t.name.clone().to_lowercase().replace(" ", "_"), t))
            .collect::<BTreeMap<String, Theme>>();

        if sub_arg.subcommand_matches("all").is_some() {
            handle_install(theme_map)?;
            return Ok(());
        } else if let Some(themes) = sub_arg.get_many::<String>("theme") {
            let themes = themes.map(Clone::clone).collect::<Vec<String>>();
            theme_map.retain(|k, _| themes.contains(k));
            handle_install(theme_map)?;
        } else {
            for (name, _) in &theme_map {
                println!("{}", name);
            }
        } 

        return Ok(());

    } else {
        handle_preprocessing(&preproc)
    }
}

fn handle_install(themes: BTreeMap<String, Theme>) -> Result<()> {
    let _ = std::fs::OpenOptions::new().read(true).create(false).open("./book.toml")?;
    if let Err(err) = std::fs::create_dir("./mdbook-code-theme") && err.kind() != std::io::ErrorKind::AlreadyExists {
        Err(err)?;
    }

    let mut css_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./mdbook-code-theme/code-themes.css")?;

    let mut theme_list= String::new(); 
    let default_theme = "ayu_dark";
    for (name, theme) in &themes {
        let theme_css= theme.to_css(&format!("html[code-theme = \"{}\"] pre>code.arborium", name));
        css_file.write(theme_css.as_bytes())?;
        theme_list.push_str(&format!("\t\t{}: \"{}\",\n", name, theme.name));
    }


    let mut js_file = std::fs::OpenOptions::new().write(true).create(true).truncate(true).open("./mdbook-code-theme/code-theme-selector.js")?;

    js_file.write(b"(function () {")?;
    write!(&mut js_file, r#"
    const available_themes = {{ 
{theme_list} 
    }};
    const default_theme = "{default_theme}";
"#)?;
    write!(&mut js_file, "{}", JS)?;

    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
    let renderer = sub_args
        .get_one::<String>("renderer")
        .expect("Required argument");
    let supported = pre.supports_renderer(renderer).unwrap();

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if supported {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> errors::Result<()> {
    let (ctx, book) = mdbook_preprocessor::parse_input(std::io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook_preprocessor::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook_preprocessor::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(std::io::stdout(), &processed_book)?;

    Ok(())
}
