pub struct SyntaxColor;

use mdbook_markdown::{
    MarkdownOptions,
    pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd, TextMergeWithOffset},
};
use mdbook_preprocessor::{
    Preprocessor, PreprocessorContext,
    book::{Book, BookItem},
    errors,
};
use std::ops::Range;

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
            let BookItem::Chapter(chapter) = item else {
                return;
            };
            let parser = TextMergeWithOffset::new(
                mdbook_markdown::new_cmark_parser(&chapter.content, &MarkdownOptions::default())
                    .into_offset_iter(),
            )
            .filter(|(e, _)| {
                matches!(
                    e,
                    Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_)))
                        | Event::End(TagEnd::CodeBlock)
                        | Event::Text(_)
                )
            });

            let mut code_blocks: Vec<CodeBlockInfo> = Vec::new();
            let mut source: Option<(Range<usize>, String)> = None;

            for (event, range) in parser {
                match event {
                    Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(code))) => {
                        if arborium::get_language(&code).is_none() {
                            continue;
                        };
                        source = Some((range, code.to_string()));
                    }
                    Event::End(TagEnd::CodeBlock) => {
                        source = None;
                    }
                    Event::Text(_) if (&source).is_some() => {
                        let (source_span, lang) = source.take().unwrap();
                        code_blocks.push(CodeBlockInfo {
                            lang,
                            source_span,
                            code_span: range,
                        });
                    }
                    Event::Text(_) => {
                        continue;
                    }
                    _ => {
                        eprintln!("{:?} : {:?}", event, range);
                        unreachable!("Iterator is filtered !");
                    }
                }
            }
            let mut new_content = String::with_capacity(chapter.content.len() * 2);
            let mut cursor = 0;
            for code_info in code_blocks {
                let Ok(highlighted) =
                    hi.highlight(&code_info.lang, &chapter.content[code_info.code_span])
                else {
                    new_content.push_str(&chapter.content[cursor..code_info.source_span.end]);
                    cursor = code_info.source_span.end;
                    continue;
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

    fn supports_renderer(&self, renderer: &str) -> anyhow::Result<bool> {
        Ok(renderer == "html" || renderer == "markdown")
    }
}
