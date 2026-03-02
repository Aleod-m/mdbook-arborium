mod preproc;

use std::{collections::BTreeMap, io::Write};

use anyhow::Result;
use arborium::theme::Theme;
use mdbook_preprocessor::{Preprocessor, errors};
use semver::{Version, VersionReq};
use thiserror::Error;

use preproc::SyntaxColor;

const JS: &str = include_str!("./theme-selector.js");

#[derive(Debug, Error)]
enum Errors {
    #[error("Theme not found: {0}")]
    ThemeNotFound(String),

    #[error("Install command expect arguments.")]
    ExpectedInstallArgument,

    #[error("Unrecognized argument: {0}")]
    UnrecognizedArgument(String),
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();
    let _bin_name = args.next();

    let preproc = SyntaxColor::new();
    let Some(arg) = args.next() else {
        // No argument we do the preprocessing.
        return handle_preprocessing(&preproc);
    };

    match arg.as_str() {
        "supports" => {
            let renderer = args.next().unwrap_or_default();
            if preproc.supports_renderer(&renderer).expect("Unfailable") {
                std::process::exit(0)
            } else {
                std::process::exit(1);
            }
        }
        "install" => {
            handle_install(args)?;
        }
        arg @ _ => return Err(Errors::UnrecognizedArgument(arg.to_string()).into()),
    }
    return Ok(());
}

/// Handle the installation of the necessary css
/// and javascript for the preprocessor to work.
fn handle_install(args: std::env::Args) -> Result<()> {
    let all_themes = arborium::theme::builtin::all()
        .into_iter()
        .map(|t| (t.name.clone().to_lowercase().replace(" ", "_"), t))
        .collect::<BTreeMap<String, Theme>>();

    let args = args.collect::<Vec<_>>();
    if args.len() < 1 {
        return Err(Errors::ExpectedInstallArgument.into());
    }

    let mut themes_to_install = Vec::<&str>::new();
    match args
        .get(0)
        .expect("Should have 1 argument at this point.")
        .as_str()
    {
        "list" => {
            println!("Available themes:");
            let _ = all_themes
                .keys()
                .for_each(|theme| println!("\t- {}", theme));
            return Ok(());
        }
        "all" => {
            themes_to_install = all_themes.keys().map(String::as_str).collect::<Vec<_>>();
        }
        _ => {
            for arg in &args {
                if all_themes.contains_key(arg) {
                    themes_to_install.push(&arg);
                } else {
                    return Err(Errors::ThemeNotFound(arg.to_string()))?;
                }
            }
        }
    }

    let _ = std::fs::OpenOptions::new()
        .read(true)
        .create(false)
        .open("./book.toml")?;

    if let Err(err) = std::fs::create_dir("./mdbook-code-theme")
        && err.kind() != std::io::ErrorKind::AlreadyExists
    {
        Err(err)?;
    }

    let mut css_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./mdbook-code-theme/code-themes.css")?;

    let mut theme_list = String::new();
    let default_theme = themes_to_install
        .first()
        .expect("At least one theme should be present at this point.");

    for theme_name in &themes_to_install {
        let theme = all_themes
            .get(*theme_name)
            .expect("Filtering the themes failed.");
        let theme_css = theme.to_css(&format!(
            "html[code-theme = \"{}\"] pre>code.arborium",
            theme_name
        ));
        css_file.write(theme_css.as_bytes())?;
        theme_list.push_str(&format!("\t\t{}: \"{}\",\n", theme_name, theme.name));
    }

    let mut js_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./mdbook-code-theme/code-theme-selector.js")?;

    js_file.write(b"(function () {")?;
    write!(
        &mut js_file,
        r#"
    const available_themes = {{ 
{theme_list} 
    }};
    const default_theme = "{default_theme}";
"#
    )?;
    write!(&mut js_file, "{}", JS)?;

    Ok(())
}

/// Run the preprocessor.
fn handle_preprocessing(preproc: &dyn Preprocessor) -> errors::Result<()> {
    let (ctx, book) = mdbook_preprocessor::parse_input(std::io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook_preprocessor::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            preproc.name(),
            mdbook_preprocessor::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = preproc.run(&ctx, book)?;
    serde_json::to_writer(std::io::stdout(), &processed_book)?;

    Ok(())
}
