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
    let descriptor = descriptor(entry);
    section(&mut out, "NAME", [entry.term.as_str(), descriptor.as_str()]);
    section(&mut out, "SYNTAX", [entry.syntax.as_str()]);
    section(&mut out, "DESCRIPTION", [entry.summary.as_str()]);

    if let Some(example) = first_faber_example(&entry.body) {
        block_section(&mut out, "EXAMPLE", example);
    }

    if !entry.related.is_empty() {
        let related = entry.related.join(", ");
        section(&mut out, "RELATED", [related.as_str()]);
    }

    if !entry.examples.is_empty() {
        let examples = entry.examples.join(", ");
        section(&mut out, "EXAMPLES", [examples.as_str()]);
    }

    out
}

fn render_legacy(entry: &Entry, canonical: &Entry) -> String {
    let mut out = String::new();
    section(&mut out, "NAME", [entry.term.as_str()]);
    section(&mut out, "STATUS", ["legacy"]);
    section(&mut out, "USE INSTEAD", [canonical.term.as_str()]);
    section(&mut out, "DESCRIPTION", [entry.summary.as_str()]);

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

fn section<I, S>(out: &mut String, title: &str, lines: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let _ = writeln!(out, "{title}");
    for line in lines {
        let _ = writeln!(out, "  {}", line.as_ref());
    }
    out.push('\n');
}

fn block_section(out: &mut String, title: &str, block: &str) {
    let _ = writeln!(out, "{title}");
    out.push_str(block.trim_end());
    out.push('\n');
    out.push('\n');
}

fn first_faber_example(body: &str) -> Option<&str> {
    let start = body.find("```fab")?;
    let code_start = body[start..].find('\n')? + start + 1;
    let code_end = body[code_start..].find("```")? + code_start;
    Some(&body[code_start..code_end])
}
