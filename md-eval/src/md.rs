use std::{
    collections::{BTreeMap, VecDeque},
    path::{Path, PathBuf},
};

use pulldown_cmark::Event;

use crate::{file2mod, Nav, Upcoming};

/// Markdown document from data folder
pub enum Document {
    /// Single markdown document, used with `#[doc = ...`, renders to .md
    Page {
        /// markdown file to generate
        name: String,
        contents: String,
        file: PathBuf,
    },
    /// Multi page document, used with `mod`, renders to .rs
    Pages {
        /// rs file to generate
        name: String,
        /// sorted alphabetically
        pages: Vec<String>,
        /// entry point to the file. "load" this to get the document again
        file: PathBuf,
        /// Original file names, used for diagnostics
        files: Vec<PathBuf>,
    },
}

// workflow:
// - inside the build script - read md, render to rust code
// - inside the runner - read md, render to md

#[derive(Debug, Clone)]
pub struct Mod {
    pub name: String,
    pub code: String,
}

impl Document {
    pub fn name(&self) -> &str {
        match self {
            Document::Page { name, .. } | Document::Pages { name, .. } => name.as_str(),
        }
    }

    pub fn ext(&self) -> &str {
        match self {
            Document::Page { .. } => "md",
            Document::Pages { .. } => "rs",
        }
    }

    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let name = file2mod(path);
        let file = path.to_owned();
        if path.is_file() {
            Ok(Self::Page {
                name,
                contents: std::fs::read_to_string(path)?,
                file,
            })
        } else {
            let mut files = Vec::new();
            for entry in path.read_dir()? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    files.push(entry.path());
                }
            }
            files.sort();
            Ok(Self::Pages {
                name,
                pages: files
                    .iter()
                    .map(std::fs::read_to_string)
                    .collect::<Result<Vec<_>, _>>()?,
                files,
                file,
            })
        }
    }
    fn read_from(&self) -> &Path {
        match self {
            Document::Page { file, .. } | Document::Pages { file, .. } => file,
        }
    }

    fn tokens(&self) -> Box<dyn Iterator<Item = (&Path, Event)> + '_> {
        use pulldown_cmark::Parser;
        match self {
            Document::Page { contents, file, .. } => {
                Box::new(std::iter::repeat(file.as_path()).zip(Parser::new(contents)))
            }
            Document::Pages { pages, files, .. } => Box::new(
                files
                    .iter()
                    .zip(pages.iter())
                    .flat_map(|(file, s)| std::iter::repeat(file.as_path()).zip(Parser::new(s))),
            ),
        }
    }

    pub fn render_rust(&self) -> anyhow::Result<Mod> {
        use pulldown_cmark::{CodeBlockKind, Tag};
        use std::fmt::Write;

        let mut fence = Upcoming::default();
        let mut modules = String::new();
        let mut execs = String::new();
        let mut mapping = BTreeMap::new();
        let mut cur_file = PathBuf::new();
        let mut ix = 0;
        for (file, t) in self.tokens() {
            if file != cur_file {
                mapping.clear();
                cur_file = file.to_owned();
            }

            match t {
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(f))) => {
                    fence = Upcoming::parse_fence(&f)?;
                    continue;
                }

                Event::Text(code) => match &fence {
                    &Upcoming::Code {
                        title: _,
                        id: Some(id),
                    } => {
                        ix += 1;
                        if mapping.insert(id, ix).is_some() {
                            anyhow::bail!("Duplicate mapping {id}");
                        }
                        writeln!(&mut modules, "mod r{ix} {{ #![allow(dead_code)]")?;
                        unhide(&mut modules, &code);
                        writeln!(&mut modules, "}}")?
                    }

                    Upcoming::Code { title: _, id: None } => {
                        ix += 1;
                        writeln!(&mut modules, "mod t{ix} {{ #![allow(dead_code)]")?;
                        unhide(&mut modules, &code);
                        writeln!(&mut modules, "}}")?
                    }
                    Upcoming::Exec { title: _, ids } => {
                        assert_eq!(ids.len(), 1);
                        let id = ids[0];
                        let code_id = mapping[&id];
                        let args = shell_words::split(&code).unwrap();
                        writeln!(
                    &mut execs,
                    "out.push(crate::render_res(r{code_id}::options().run_inner(&{args:?})));"
                )?;
                    }
                    Upcoming::Ignore => {}
                },
                _ => {}
            }
            fence = Upcoming::Ignore;
        }

        let read_from = self.read_from();
        let code = format!(
            "mod {name} {{
        {modules}
        pub fn run(output_dir: &std::path::Path) {{
            #[allow(unused_mut)]
            let mut out = Vec::new();
            {execs}

            let doc = md_eval::md::Document::load({read_from:?}).expect(\"Failed to read \\{read_from:?});
            let md = doc.render_markdown(&out).expect(\"Failed to render \\{read_from:?});

            let dest = std::path::PathBuf::from(output_dir).join(\"{name}.{ext}\");
            std::fs::write(dest, md.to_string()).unwrap();

        }}
    }}",
            name = self.name(),
            ext = self.ext(),
        );
        let name = self.name().to_string();
        Ok(Mod { name, code })
    }

    pub fn render_markdown(&self, out: &[String]) -> anyhow::Result<String> {
        let mut res = String::new();

        match self {
            Document::Page { contents, .. } => {
                let mut x = 0;
                let events = splice_output(contents, out, &mut x);
                pulldown_cmark_to_cmark::cmark(events, &mut res)?;
                Ok(res)
            }
            Document::Pages { pages, name, .. } => {
                use std::fmt::Write;
                let mut x = 0;

                let nav = Nav {
                    pad: "//!",
                    prev: None,
                    index: None,
                    next: (pages.len() > 1).then_some("page_1"),
                };

                let mut buf = String::new();
                pulldown_cmark_to_cmark::cmark(splice_output(&pages[0], out, &mut x), &mut buf)?;

                for line in buf.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        writeln!(&mut res, "//!")?;
                    } else {
                        writeln!(&mut res, "//! {line}")?;
                    }
                }

                write!(&mut res, "{nav}")?;

                writeln!(
                    &mut res,
                    "#[allow(unused_imports)] use crate::{{*, parsers::*}};"
                )?;

                let index_link = format!("super::{name}");
                for (page, child) in pages[1..].iter().enumerate() {
                    let page = page + 1;
                    let prev_page = format!("page_{}", page - 1);
                    let next_page = format!("page_{}", page + 1);
                    buf.clear();

                    let mut buf = String::new();
                    pulldown_cmark_to_cmark::cmark(splice_output(child, out, &mut x), &mut buf)?;

                    writeln!(&mut res, "\n")?;
                    for line in buf.lines() {
                        let line = line.trim();
                        if line.is_empty() {
                            writeln!(&mut res, "///")?;
                        } else {
                            writeln!(&mut res, "/// {line}")?;
                        }
                    }

                    let nav = Nav {
                        pad: "///",
                        prev: Some(if page == 1 { &index_link } else { &prev_page }),
                        index: Some(&index_link),
                        next: (page < pages.len() - 1).then_some(&next_page),
                    };

                    write!(&mut res, "{nav}")?;

                    writeln!(&mut res, "pub mod page_{page} {{}}")?;
                }

                Ok(res)
            } //Ok(format!("hello world {out:?}")),
        }
    }
}

fn unhide(f: &mut String, code: &str) {
    for line in code.lines() {
        *f += line.strip_prefix("# ").unwrap_or(line);
        *f += "\n"
    }
}

fn splice_output<'a>(
    contents: &'a str,
    outs: &'a [String],
    outs_used: &'a mut usize,
) -> impl Iterator<Item = Event<'a>> {
    Splicer {
        contents: pulldown_cmark::Parser::new(contents),
        outs,
        outs_used,
        queue: VecDeque::new(),
    }
}

struct Splicer<'a, I> {
    contents: I,
    outs: &'a [String],
    outs_used: &'a mut usize,
    queue: VecDeque<Event<'a>>,
}

const STYLE: &str = "padding: 14px; background-color:var(--code-block-background-color); font-family: 'Source Code Pro', monospace; margin-bottom: 0.75em;";
impl<'a, I: Iterator<Item = Event<'a>>> Iterator for Splicer<'a, I> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        use pulldown_cmark::{CodeBlockKind, Tag};
        if let Some(q) = self.queue.pop_front() {
            return Some(q);
        }

        match self.contents.next()? {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(f))) => {
                let fence = Upcoming::parse_fence(&f).unwrap();
                match &fence {
                    Upcoming::Code { title, .. } => {
                        if let Some(title) = title {
                            self.queue.push_back(fold_open(title.as_str()));
                        }
                        let cb = Tag::CodeBlock(CodeBlockKind::Fenced("rust".into()));
                        self.queue.push_back(Event::Start(cb));
                        // transfer rust code text and closing codeblock tag
                        self.queue.push_back(self.contents.next()?);
                        self.queue.push_back(self.contents.next()?);
                        if title.is_some() {
                            self.queue.push_back(Event::Html("</details>".into()));
                        }

                        self.queue.pop_front()
                    }
                    Upcoming::Exec { title, .. } => {
                        if let Some(title) = title {
                            self.queue.push_back(fold_open(title));
                        }

                        let Event::Text(code) = self.contents.next()? else {
                            panic!();
                        };
                        let snip = &self.outs[*self.outs_used];
                        let html = format!(
                            "\n\n<div style={STYLE:?}>\n$ app {code}<br />\n\n{snip}\n\n</div>\n\n"
                        );
                        self.queue.push_back(Event::Html(html.into()));
                        *self.outs_used += 1;

                        self.contents.next()?;

                        if title.is_some() {
                            self.queue.push_back(Event::Html("</details>".into()));
                        }
                        self.queue.pop_front()
                    }
                    _ => Some(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(f)))),
                }
            }
            event => Some(event),
        }
    }
}

fn fold_open(title: &str) -> Event<'static> {
    Event::Html(format!("<details><summary>{title}</summary>").into())
}
