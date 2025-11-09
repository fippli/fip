use pulldown_cmark::{html, Event, HeadingLevel, Options, Parser, Tag};
use std::{
    borrow::Cow,
    collections::HashMap,
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Debug)]
struct DocPage {
    title: String,
    source_path: PathBuf,
    content_html: String,
    section_id: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let project_root = project_root()?;
    let syntax_dir = project_root.join("syntax");
    let docs_dir = project_root.join("docs");

    if !syntax_dir.exists() {
        return Err(format!("syntax directory not found at {}", syntax_dir.display()).into());
    }

    fs::create_dir_all(&docs_dir)?;

    let markdown_files = collect_markdown(&syntax_dir)?;
    if markdown_files.is_empty() {
        return Err("no markdown files found in /syntax".into());
    }

    let mut pages = Vec::new();
    for path in markdown_files {
        let content = fs::read_to_string(&path)?;
        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("invalid file name {}", path.display()))?;
        let slug_prefix = file_stem.replace('_', "-");
        let section_id = format!("section-{}", slug_prefix);
        let (html, doc_title) = render_markdown(&content, &slug_prefix);
        pages.push(DocPage {
            title: doc_title.unwrap_or_else(|| humanize_stem(file_stem)),
            source_path: path,
            content_html: html,
            section_id,
        });
    }

    pages.sort_by(|a, b| a.title.cmp(&b.title));

    cleanup_existing_html(&docs_dir)?;

    let index_html = build_full_site_html(&pages)?;
    fs::write(docs_dir.join("index.html"), index_html)?;

    Ok(())
}

fn cleanup_existing_html(docs_dir: &Path) -> Result<(), Box<dyn Error>> {
    if docs_dir.exists() {
        for entry in fs::read_dir(docs_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("html") {
                fs::remove_file(path)?;
            }
        }
    }
    Ok(())
}

fn project_root() -> Result<PathBuf, Box<dyn Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut ancestors = manifest_dir.ancestors();
    // ancestors(): nth(0) self, 1 parent, 2 grandparent, etc.
    let root = ancestors
        .nth(3)
        .ok_or("could not determine project root from manifest path")?;
    Ok(root.to_path_buf())
}

fn collect_markdown(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.into_path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn render_markdown(markdown: &str, slug_prefix: &str) -> (String, Option<String>) {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);
    let mut events: Vec<Event<'_>> = parser.collect();
    let mut headings = Vec::new();
    let mut slug_counts: HashMap<String, usize> = HashMap::new();

    let mut i = 0usize;
    while i < events.len() {
        if let Event::Start(Tag::Heading(level, _, _)) = events[i].clone() {
            let (title, end_index) = collect_heading_text(&events, i + 1);
            let mut slug = slugify(&title);
            let counter = slug_counts.entry(slug.clone()).or_insert(0);
            if *counter > 0 {
                slug = format!("{}-{}", slug, counter);
            }
            *counter += 1;

            slug = format!("{}-{}", slug_prefix, slug);

            let level_num = heading_level_to_u8(&level);
            let slug_for_id = slug.clone();
            events[i] = Event::Html(
                format!(
                    "<h{lvl} id=\"{slug}\">",
                    lvl = level_num,
                    slug = slug_for_id
                )
                .into(),
            );
            if let Event::End(Tag::Heading(_, _, _)) = events[end_index].clone() {
                events[end_index] = Event::Html(format!("</h{lvl}>", lvl = level_num).into());
            }

            headings.push((level, slug, title));
            i = end_index;
        }
        i += 1;
    }

    let mut html_output = String::new();
    html::push_html(&mut html_output, events.into_iter());

    let mut doc_title = None;
    for (level, _slug, title) in headings {
        let level_num = heading_level_to_u8(&level);
        if level_num == 1 && doc_title.is_none() {
            doc_title = Some(title.clone());
        }
    }

    (html_output, doc_title)
}

fn collect_heading_text(events: &[Event<'_>], mut index: usize) -> (String, usize) {
    let mut text = String::new();
    while index < events.len() {
        match &events[index] {
            Event::End(Tag::Heading(_, _, _)) => break,
            Event::Text(content) | Event::Code(content) => {
                text.push_str(content);
            }
            Event::Html(content) => {
                text.push_str(content);
            }
            _ => {}
        }
        index += 1;
    }
    (text.trim().to_string(), index)
}

fn heading_level_to_u8(level: &HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn slugify(title: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if ch.is_whitespace() || ch == '-' || ch == '_' {
            if !last_dash && !slug.is_empty() {
                slug.push('-');
                last_dash = true;
            }
        }
    }
    if slug.is_empty() {
        "section".to_string()
    } else if slug.ends_with('-') {
        slug.trim_end_matches('-').to_string()
    } else {
        slug
    }
}

fn humanize_stem(stem: &str) -> String {
    stem.split('-')
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => {
                    let mut word = first.to_ascii_uppercase().to_string();
                    word.push_str(chars.as_str());
                    word
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn build_full_site_html(pages: &[DocPage]) -> Result<String, Box<dyn Error>> {
    let mut sections_html = String::new();
    for page in pages {
        let source_path_display = page.source_path.to_string_lossy();
        let source = html_escape(&source_path_display);
        sections_html.push_str(&format!(
            "<section id=\"{id}\" data-doc-section=\"{id}\" data-source=\"{source}\">\n{content}\n</section>\n",
            id = page.section_id,
            source = source,
            content = page.content_html
        ));
    }

    let html = format!(
        r##"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Fip Language Documentation</title>
    <link
      rel="stylesheet"
      href="https://cdn.jsdelivr.net/gh/fippli/css@latest/regular.css"
    />
  </head>
  <body id="top">
    <main>
      {sections}
    </main>
  </body>
</html>
"##,
        sections = sections_html,
    );

    Ok(html)
}

fn html_escape(input: &str) -> Cow<'_, str> {
    if input.contains(['<', '>', '&', '"', '\'']) {
        Cow::Owned(
            input
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
                .replace('\'', "&#39;"),
        )
    } else {
        Cow::Borrowed(input)
    }
}
