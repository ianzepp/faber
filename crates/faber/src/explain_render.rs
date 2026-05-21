use crate::explain::{Entry, Lookup, SearchHit};
use std::fmt::Write;

pub fn render_lookup_plain(lookup: &Lookup<'_>) -> String {
    match lookup {
        Lookup::Exact(entry) | Lookup::Alias { entry, .. } => render_canonical(entry),
        Lookup::Legacy {
            entry, canonical, ..
        } => render_legacy(entry, canonical),
    }
}

pub fn render_search(query: &str, hits: &[SearchHit<'_>]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Search: {query}");

    if hits.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(out, "No matches.");
        return out;
    }

    let _ = writeln!(out);
    for hit in hits {
        let entry = hit.entry;
        let _ = writeln!(
            out,
            "{}\t{} / {}\t{}",
            entry.term,
            entry.kind.as_str(),
            entry.category,
            entry.summary
        );
    }

    out
}

fn render_canonical(entry: &Entry) -> String {
    let mut out = String::new();
    header(&mut out, &entry.term);
    let name = name_line(&entry.term, &entry.summary);
    section(&mut out, "NAME", [name.as_str()]);
    let descriptor = descriptor(entry);
    section(&mut out, "KIND", [descriptor.as_str()]);
    section(&mut out, "SYNTAX", [entry.syntax.as_str()]);
    let description = description_text(entry);
    section(&mut out, "DESCRIPTION", [description.as_str()]);

    if let Some(example) = first_faber_example(&entry.body) {
        block_section(&mut out, "EXAMPLE", example);
    }

    if !entry.related.is_empty() {
        let related = entry.related.join(", ");
        section(&mut out, "RELATED", [related.as_str()]);
    }

    out
}

fn render_legacy(entry: &Entry, canonical: &Entry) -> String {
    let mut out = String::new();
    header(&mut out, &entry.term);
    let name = format!("{} - legacy spelling for {}", entry.term, canonical.term);
    section(&mut out, "NAME", [name.as_str()]);
    section(&mut out, "STATUS", ["Legacy. Not canonical Faber source."]);
    section(&mut out, "USE INSTEAD", [canonical.term.as_str()]);
    let description = description_text(entry);
    section(&mut out, "DESCRIPTION", [description.as_str()]);

    if let Some(example) = first_faber_example(&entry.body) {
        block_section(&mut out, "EXAMPLE", example);
    }

    if !entry.related.is_empty() {
        let related = entry.related.join(", ");
        section(&mut out, "SEE ALSO", [related.as_str()]);
    }

    out
}

fn descriptor(entry: &Entry) -> String {
    format!("{} / {}", entry.kind.as_str(), entry.category)
}

fn header(out: &mut String, term: &str) {
    let page = format!("{}(7)", term.to_uppercase());
    let center = "Faber Language Reference";
    let width = 78usize;
    let fixed = page.len() * 2 + center.len();
    if fixed + 2 > width {
        let _ = writeln!(out, "{page}  {center}");
        out.push('\n');
        return;
    }

    let spaces = width - fixed;
    let left_spaces = spaces / 2;
    let right_spaces = spaces - left_spaces;
    let _ = writeln!(
        out,
        "{}{}{}{}{}",
        page,
        " ".repeat(left_spaces),
        center,
        " ".repeat(right_spaces),
        page
    );
    out.push('\n');
}

fn name_line(term: &str, summary: &str) -> String {
    format!("{} - {}", term, sentence_fragment(summary))
}

fn sentence_fragment(summary: &str) -> String {
    let summary = summary.trim().trim_end_matches('.');
    let mut chars = summary.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    first.to_lowercase().chain(chars).collect()
}

fn description_text(entry: &Entry) -> String {
    let prose = body_without_code_blocks(&entry.body);
    if prose.trim().is_empty() {
        return entry.summary.clone();
    }
    prose
}

fn body_without_code_blocks(body: &str) -> String {
    let mut out = String::new();
    let mut in_code = false;

    for line in body.lines() {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(line.trim_end());
    }

    out.trim().to_owned()
}

fn section<I, S>(out: &mut String, title: &str, lines: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let _ = writeln!(out, "{title}");
    for line in lines {
        for part in line.as_ref().lines() {
            let _ = writeln!(out, "    {part}");
        }
    }
    out.push('\n');
}

fn block_section(out: &mut String, title: &str, block: &str) {
    let _ = writeln!(out, "{title}");
    let block = normalize_faber_block(block);
    for line in block.lines() {
        let _ = writeln!(out, "    {}", line);
    }
    out.push('\n');
}

fn normalize_faber_block(block: &str) -> String {
    let mut out = String::new();
    let mut depth = 0usize;

    for raw_line in block.trim().lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            out.push('\n');
            continue;
        }

        if line.starts_with('}') {
            depth = depth.saturating_sub(1);
        }

        out.push_str(&"    ".repeat(depth));
        out.push_str(line);
        out.push('\n');

        let opens = line.chars().filter(|ch| *ch == '{').count();
        let closes = line.chars().filter(|ch| *ch == '}').count();
        depth = depth.saturating_add(opens).saturating_sub(closes);
    }

    out.trim_end().to_owned()
}

fn first_faber_example(body: &str) -> Option<&str> {
    let start = body.find("```fab")?;
    let code_start = body[start..].find('\n')? + start + 1;
    let code_end = body[code_start..].find("```")? + code_start;
    Some(&body[code_start..code_end])
}
