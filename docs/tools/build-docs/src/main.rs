use pulldown_cmark::{html, Event, HeadingLevel, Options, Parser, Tag};
use std::{
    borrow::Cow,
    cmp::Ordering,
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
    h1_slug: String,
    h2_headings: Vec<(String, String)>, // (slug, title)
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

    let spec_order = load_spec_order(&syntax_dir)?;

    let mut pages = Vec::new();
    for path in markdown_files {
        // Skip index.md - it's only used for ordering, not content
        if path.file_name().and_then(|n| n.to_str()) == Some("index.md") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("invalid file name {}", path.display()))?;
        let slug_prefix = file_stem.replace('_', "-");
        let section_id = format!("section-{}", slug_prefix);
        let (html, doc_title, h1_slug, h2_headings) = render_markdown(&content, &slug_prefix);
        let title = doc_title
            .clone()
            .unwrap_or_else(|| humanize_stem(file_stem));
        let fallback_slug = format!("{}-{}", slug_prefix, slugify(&title));
        pages.push(DocPage {
            title,
            source_path: path,
            content_html: html,
            section_id,
            h1_slug: h1_slug.unwrap_or(fallback_slug),
            h2_headings,
        });
    }

    let syntax_dir_for_sort = syntax_dir.clone();
    pages.sort_by(|a, b| page_order(a, b, &syntax_dir_for_sort, &spec_order));

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

fn load_spec_order(syntax_dir: &Path) -> Result<HashMap<PathBuf, usize>, Box<dyn Error>> {
    let mut order_map = HashMap::new();

    let index_path = syntax_dir.join("index.md");
    if !index_path.exists() {
        return Ok(order_map);
    }

    let content = fs::read_to_string(index_path)?;
    let mut position = 1usize;

    for line in content.lines() {
        let trimmed = line.trim();
        // Parse lines like "1. `./overview.md`" or "11. `./test.md`"
        // Look for backticked paths
        for (idx, segment) in trimmed.split('`').enumerate() {
            if idx % 2 != 1 {
                continue;
            }
            let path_segment = segment.trim();
            if path_segment.starts_with("./") && path_segment.ends_with(".md") {
                let rel = path_segment.trim_start_matches("./");
                let rel_path = PathBuf::from(rel);
                if !order_map.contains_key(&rel_path) {
                    order_map.insert(rel_path, position);
                    position += 1;
                }
            }
        }
    }

    Ok(order_map)
}

fn render_markdown(
    markdown: &str,
    slug_prefix: &str,
) -> (
    String,
    Option<String>,
    Option<String>,
    Vec<(String, String)>,
) {
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

    // Remove class attributes from code elements
    html_output = strip_code_classes(&html_output);

    let mut doc_title = None;
    let mut h1_slug = None;
    let mut h2_headings = Vec::new();
    for (level, slug, title) in headings {
        let level_num = heading_level_to_u8(&level);
        if level_num == 1 && doc_title.is_none() {
            doc_title = Some(title.clone());
            h1_slug = Some(slug.clone());
        } else if level_num == 2 {
            h2_headings.push((slug, title));
        }
    }

    (html_output, doc_title, h1_slug, h2_headings)
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

    // Build sidebar navigation from H1 headings with nested H2 headings
    let mut sidebar_items = String::new();
    for page in pages {
        let title_escaped = html_escape(&page.title);
        sidebar_items.push_str(&format!(
            "      <li data-nav-item>\n        <a href=\"#{slug}\">{title}</a>\n",
            slug = page.h1_slug,
            title = title_escaped
        ));

        // Add H2 headings as nested list
        if !page.h2_headings.is_empty() {
            sidebar_items.push_str("        <ul>\n");
            for (h2_slug, h2_title) in &page.h2_headings {
                let h2_title_escaped = html_escape(h2_title);
                sidebar_items.push_str(&format!(
                    "          <li data-nav-item><a href=\"#{slug}\">{title}</a></li>\n",
                    slug = h2_slug,
                    title = h2_title_escaped
                ));
            }
            sidebar_items.push_str("        </ul>\n");
        }
        sidebar_items.push_str("      </li>\n");
    }

    let sidebar_html = format!(
        r##"    <nav>
      <input type="text" id="nav-filter" placeholder="Filter headings..." />
      <ul id="nav-list">
{items}      </ul>
    </nav>
    <script>
      (function() {{
        const filterInput = document.getElementById('nav-filter');
        const navList = document.getElementById('nav-list');
        const navItems = navList.querySelectorAll('[data-nav-item]');
        
        filterInput.addEventListener('input', function(e) {{
          const filter = e.target.value.toLowerCase().trim();
          
          navItems.forEach(function(item) {{
            const link = item.querySelector('a');
            if (!link) return;
            
            const text = link.textContent.toLowerCase();
            const matches = text.includes(filter);
            
            if (matches) {{
              item.style.display = '';
              // Show parent H1 if H2 matches
              const parentLi = item.closest('li[data-nav-item]');
              if (parentLi && parentLi !== item) {{
                parentLi.style.display = '';
              }}
            }} else {{
              item.style.display = filter === '' ? '' : 'none';
            }}
          }});
          
          // Hide H1 items if all their H2 children are hidden
          navList.querySelectorAll('> li[data-nav-item]').forEach(function(h1Item) {{
            const h2List = h1Item.querySelector('ul');
            if (h2List) {{
              const visibleH2s = Array.from(h2List.querySelectorAll('li[data-nav-item]'))
                .filter(function(li) {{ return li.style.display !== 'none'; }});
              const hasVisibleH2s = visibleH2s.length > 0;
              const h1Link = h1Item.querySelector('> a');
              const h1Matches = h1Link && h1Link.textContent.toLowerCase().includes(filter);
              
              if (!hasVisibleH2s && !h1Matches && filter !== '') {{
                h1Item.style.display = 'none';
              }} else {{
                h1Item.style.display = '';
              }}
            }}
          }});
        }});
      }})();
    </script>"##,
        items = sidebar_items
    );

    let html = format!(
        r##"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Fip Language Documentation</title>
    <link rel="stylesheet" href="style.css" />
  </head>
  <body id="top">
{sidebar}
    <main>
      {sections}
    </main>
  </body>
</html>
"##,
        sidebar = sidebar_html,
        sections = sections_html,
    );

    Ok(html)
}

fn page_order(
    a: &DocPage,
    b: &DocPage,
    syntax_dir: &Path,
    order_map: &HashMap<PathBuf, usize>,
) -> Ordering {
    let rel_a = relative_to_syntax(&a.source_path, syntax_dir);
    let rel_b = relative_to_syntax(&b.source_path, syntax_dir);

    let key_a = order_map.get(&rel_a);
    let key_b = order_map.get(&rel_b);

    match (key_a, key_b) {
        (Some(pos_a), Some(pos_b)) => pos_a.cmp(pos_b),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.title.cmp(&b.title),
    }
}

fn relative_to_syntax(path: &Path, syntax_dir: &Path) -> PathBuf {
    path.strip_prefix(syntax_dir)
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|_| path.to_path_buf())
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

fn strip_code_classes(html: &str) -> String {
    // Remove class attributes from <code> tags using simple string replacement
    // Pattern: <code class="..."> -> <code>
    let mut result = html.to_string();

    // Find and replace <code class="..." patterns
    let mut search_pos = 0;
    while let Some(code_start) = result[search_pos..].find("<code") {
        let code_start_abs = search_pos + code_start;
        if let Some(class_start) = result[code_start_abs..].find(" class=\"") {
            let class_start_abs = code_start_abs + class_start;
            // Find the closing quote
            if let Some(quote_end) = result[class_start_abs + 8..].find('"') {
                let quote_end_abs = class_start_abs + 8 + quote_end;
                // Remove the class attribute (including the space before it)
                result.replace_range(class_start_abs..=quote_end_abs, "");
                search_pos = code_start_abs;
            } else {
                search_pos = code_start_abs + 1;
            }
        } else {
            search_pos = code_start_abs + 5;
        }
    }

    result
}
