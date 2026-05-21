use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy)]
pub struct RawEntry {
    pub filename: &'static str,
    pub source: &'static str,
}

pub static RAW_ENTRIES: &[RawEntry] = include!(concat!(env!("OUT_DIR"), "/explain_entries.rs"));

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entry {
    pub term: String,
    pub kind: Kind,
    pub category: String,
    pub canonical: bool,
    pub summary: String,
    pub syntax: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub legacy: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_term: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<String>,
    pub body: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    Keyword,
    Operator,
    Annotation,
    Type,
    Modifier,
    Legacy,
    Concept,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lookup<'a> {
    Exact(&'a Entry),
    Alias {
        query: String,
        entry: &'a Entry,
    },
    Legacy {
        query: String,
        entry: &'a Entry,
        canonical: &'a Entry,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registry {
    entries: Vec<Entry>,
    terms: BTreeMap<String, usize>,
    aliases: BTreeMap<String, usize>,
    legacy: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplainError {
    pub message: String,
}

impl ExplainError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ExplainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ExplainError {}

impl Registry {
    pub fn load() -> Result<Self, ExplainError> {
        Self::from_raw_entries(RAW_ENTRIES)
    }

    pub fn from_raw_entries(raw_entries: &[RawEntry]) -> Result<Self, ExplainError> {
        let mut entries = Vec::new();
        for raw in raw_entries {
            entries.push(parse_entry(raw.filename, raw.source)?);
        }

        let mut registry = Self {
            entries,
            terms: BTreeMap::new(),
            aliases: BTreeMap::new(),
            legacy: BTreeMap::new(),
        };
        registry.index()?;
        registry.validate_references()?;
        Ok(registry)
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn categories(&self) -> BTreeSet<&str> {
        self.entries
            .iter()
            .map(|entry| entry.category.as_str())
            .collect()
    }

    pub fn by_category(&self, category: &str) -> Vec<&Entry> {
        self.entries
            .iter()
            .filter(|entry| entry.category == category)
            .collect()
    }

    pub fn lookup(&self, query: &str) -> Option<Lookup<'_>> {
        if let Some(index) = self.terms.get(query) {
            let entry = &self.entries[*index];
            if entry.canonical {
                return Some(Lookup::Exact(entry));
            }

            let canonical_term = entry.canonical_term.as_deref()?;
            let canonical = self
                .terms
                .get(canonical_term)
                .map(|index| &self.entries[*index])?;
            return Some(Lookup::Legacy {
                query: query.to_owned(),
                entry,
                canonical,
            });
        }

        if let Some(index) = self.aliases.get(query) {
            return Some(Lookup::Alias {
                query: query.to_owned(),
                entry: &self.entries[*index],
            });
        }

        let legacy_index = self.legacy.get(query)?;
        let entry = &self.entries[*legacy_index];
        let canonical_term = entry.canonical_term.as_deref()?;
        let canonical = self
            .terms
            .get(canonical_term)
            .map(|index| &self.entries[*index])?;
        Some(Lookup::Legacy {
            query: query.to_owned(),
            entry,
            canonical,
        })
    }

    fn index(&mut self) -> Result<(), ExplainError> {
        for (index, entry) in self.entries.iter().enumerate() {
            insert_unique(&mut self.terms, &entry.term, index, "term")?;
        }

        for (index, entry) in self.entries.iter().enumerate() {
            for alias in &entry.aliases {
                insert_unique(&mut self.aliases, alias, index, "alias")?;
            }
        }

        for (index, entry) in self.entries.iter().enumerate() {
            if entry.canonical {
                continue;
            }

            insert_unique(&mut self.legacy, &entry.term, index, "legacy term")?;
        }

        for (index, entry) in self.entries.iter().enumerate() {
            for legacy in &entry.legacy {
                if let Some(explicit_index) = self.terms.get(legacy) {
                    if !self.entries[*explicit_index].canonical {
                        continue;
                    }
                }

                insert_unique(&mut self.legacy, legacy, index, "legacy alias")?;
            }
        }

        Ok(())
    }

    fn validate_references(&self) -> Result<(), ExplainError> {
        for entry in &self.entries {
            if !entry.canonical && entry.canonical_term.is_none() {
                return Err(ExplainError::new(format!(
                    "{} is non-canonical but has no canonical_term",
                    entry.term
                )));
            }

            if let Some(canonical_term) = &entry.canonical_term {
                let Some(index) = self.terms.get(canonical_term) else {
                    return Err(ExplainError::new(format!(
                        "{} points to missing canonical_term {canonical_term}",
                        entry.term
                    )));
                };
                if !self.entries[*index].canonical {
                    return Err(ExplainError::new(format!(
                        "{} points to non-canonical canonical_term {canonical_term}",
                        entry.term
                    )));
                }
            }

            for related in &entry.related {
                if !self.terms.contains_key(related)
                    && !self.aliases.contains_key(related)
                    && !self.legacy.contains_key(related)
                {
                    return Err(ExplainError::new(format!(
                        "{} has unknown related term {related}",
                        entry.term
                    )));
                }
            }
        }

        Ok(())
    }
}

pub fn render_plain(lookup: &Lookup<'_>) -> String {
    match lookup {
        Lookup::Exact(entry) | Lookup::Alias { entry, .. } => render_entry(entry),
        Lookup::Legacy {
            entry, canonical, ..
        } => render_legacy(entry, canonical),
    }
}

pub fn render_json(lookup: &Lookup<'_>) -> Result<String, ExplainError> {
    match lookup {
        Lookup::Exact(entry) => serde_json::to_string_pretty(entry),
        Lookup::Alias { query, entry } => serde_json::to_string_pretty(&JsonLookup {
            query,
            resolved_via: "alias",
            entry,
            canonical: None,
        }),
        Lookup::Legacy {
            query,
            entry,
            canonical,
        } => serde_json::to_string_pretty(&JsonLookup {
            query,
            resolved_via: "legacy",
            entry,
            canonical: Some(canonical),
        }),
    }
    .map_err(|err| ExplainError::new(format!("failed to render JSON: {err}")))
}

pub fn render_list(registry: &Registry) -> String {
    let mut out = String::new();
    for entry in registry.entries() {
        if entry.canonical {
            out.push_str(&format!(
                "{}\t{}\t{}\n",
                entry.term,
                entry.kind.as_str(),
                entry.category
            ));
        }
    }
    out
}

pub fn render_category(registry: &Registry, category: &str) -> Option<String> {
    let entries = registry.by_category(category);
    if entries.is_empty() {
        return None;
    }

    let mut out = String::new();
    for entry in entries {
        out.push_str(&format!("{}\t{}\n", entry.term, entry.summary));
    }
    Some(out)
}

#[derive(Serialize)]
struct JsonLookup<'a> {
    query: &'a str,
    resolved_via: &'static str,
    entry: &'a Entry,
    #[serde(skip_serializing_if = "Option::is_none")]
    canonical: Option<&'a Entry>,
}

fn render_entry(entry: &Entry) -> String {
    let mut out = String::new();
    out.push_str(&entry.term);
    out.push('\n');
    out.push_str(&format!("Kind: {}\n", entry.kind.as_str()));
    out.push_str(&format!("Category: {}\n", entry.category));
    out.push_str(&format!("Meaning: {}\n", entry.summary));

    if let Some(example) = first_faber_example(&entry.body) {
        out.push_str("\nExample:\n");
        out.push_str(example.trim());
        out.push('\n');
    }

    out.push_str(&format!("\nSyntax: {}\n", entry.syntax));
    if !entry.examples.is_empty() {
        out.push_str(&format!("Examples: {}\n", entry.examples.join(", ")));
    }
    if !entry.related.is_empty() {
        out.push_str(&format!("Related: {}\n", entry.related.join(", ")));
    }
    out
}

fn render_legacy(entry: &Entry, canonical: &Entry) -> String {
    let mut out = String::new();
    out.push_str(&entry.term);
    out.push('\n');
    out.push_str("Legacy: not canonical Faber source.\n");
    out.push_str(&format!("Use: {}\n", canonical.term));
    out.push_str(&format!("Meaning: {}\n", entry.summary));

    if let Some(example) = first_faber_example(&entry.body) {
        out.push_str("\nExample:\n");
        out.push_str(example.trim());
        out.push('\n');
    }

    out.push_str(&format!("\nSyntax: {}\n", entry.syntax));
    if !entry.related.is_empty() {
        out.push_str(&format!("Related: {}\n", entry.related.join(", ")));
    }
    out
}

fn first_faber_example(body: &str) -> Option<&str> {
    let start = body.find("```fab")?;
    let code_start = body[start..].find('\n')? + start + 1;
    let code_end = body[code_start..].find("```")? + code_start;
    Some(&body[code_start..code_end])
}

fn parse_entry(filename: &str, source: &str) -> Result<Entry, ExplainError> {
    let mut lines = source.lines();
    if lines.next() != Some("---") {
        return Err(ExplainError::new(format!(
            "{filename}: missing frontmatter"
        )));
    }

    let mut frontmatter = Vec::new();
    let mut body_start = None;
    let mut offset = 4;
    for line in source[4..].split_inclusive('\n') {
        let trimmed = line.trim_end_matches('\n');
        if trimmed == "---" {
            body_start = Some(offset + line.len());
            break;
        }
        frontmatter.push(trimmed.trim_end_matches('\r').to_owned());
        offset += line.len();
    }

    let Some(body_start) = body_start else {
        return Err(ExplainError::new(format!(
            "{filename}: unterminated frontmatter"
        )));
    };

    let body = source[body_start..].trim().to_owned();
    if body.is_empty() {
        return Err(ExplainError::new(format!(
            "{filename}: body must not be empty"
        )));
    }
    if first_faber_example(&body).is_none() {
        return Err(ExplainError::new(format!(
            "{filename}: body must contain a fab code example"
        )));
    }

    let map = parse_frontmatter(filename, &frontmatter)?;
    let entry = Entry {
        term: required_string(filename, &map, "term")?,
        kind: parse_kind(filename, &required_string(filename, &map, "kind")?)?,
        category: required_string(filename, &map, "category")?,
        canonical: required_bool(filename, &map, "canonical")?,
        summary: required_string(filename, &map, "summary")?,
        syntax: required_string(filename, &map, "syntax")?,
        examples: optional_list(&map, "examples"),
        aliases: optional_list(&map, "aliases"),
        legacy: optional_list(&map, "legacy"),
        canonical_term: optional_string(&map, "canonical_term"),
        related: optional_list(&map, "related"),
        body,
    };

    validate_entry(filename, &entry)?;
    Ok(entry)
}

fn parse_frontmatter(
    filename: &str,
    lines: &[String],
) -> Result<BTreeMap<String, FrontValue>, ExplainError> {
    let allowed = [
        "term",
        "kind",
        "category",
        "canonical",
        "summary",
        "syntax",
        "examples",
        "aliases",
        "legacy",
        "canonical_term",
        "related",
    ];
    let mut map = BTreeMap::new();
    let mut current_list: Option<String> = None;

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }

        if let Some(item) = line.strip_prefix("  - ") {
            let Some(key) = current_list.as_ref() else {
                return Err(ExplainError::new(format!(
                    "{filename}: list item without list key"
                )));
            };
            match map.get_mut(key) {
                Some(FrontValue::List(items)) => items.push(parse_scalar(item)),
                _ => {
                    return Err(ExplainError::new(format!(
                        "{filename}: invalid list state for {key}"
                    )))
                }
            }
            continue;
        }

        let Some((key, value)) = line.split_once(':') else {
            return Err(ExplainError::new(format!(
                "{filename}: invalid frontmatter line {line:?}"
            )));
        };
        let key = key.trim().to_owned();
        if !allowed.contains(&key.as_str()) {
            return Err(ExplainError::new(format!(
                "{filename}: unknown frontmatter field {key}"
            )));
        }
        if map.contains_key(&key) {
            return Err(ExplainError::new(format!(
                "{filename}: duplicate frontmatter field {key}"
            )));
        }

        let value = value.trim();
        if value.is_empty() {
            map.insert(key.clone(), FrontValue::List(Vec::new()));
            current_list = Some(key);
        } else {
            map.insert(key, FrontValue::Scalar(parse_scalar(value)));
            current_list = None;
        }
    }

    Ok(map)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FrontValue {
    Scalar(String),
    List(Vec<String>),
}

fn parse_scalar(value: &str) -> String {
    let value = value.trim();
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        value[1..value.len() - 1].to_owned()
    } else {
        value.to_owned()
    }
}

fn required_string(
    filename: &str,
    map: &BTreeMap<String, FrontValue>,
    key: &str,
) -> Result<String, ExplainError> {
    match map.get(key) {
        Some(FrontValue::Scalar(value)) if !value.is_empty() => Ok(value.clone()),
        Some(_) => Err(ExplainError::new(format!(
            "{filename}: {key} must be a non-empty string"
        ))),
        None => Err(ExplainError::new(format!(
            "{filename}: missing required field {key}"
        ))),
    }
}

fn optional_string(map: &BTreeMap<String, FrontValue>, key: &str) -> Option<String> {
    match map.get(key) {
        Some(FrontValue::Scalar(value)) if !value.is_empty() => Some(value.clone()),
        _ => None,
    }
}

fn optional_list(map: &BTreeMap<String, FrontValue>, key: &str) -> Vec<String> {
    match map.get(key) {
        Some(FrontValue::List(items)) => items.clone(),
        Some(FrontValue::Scalar(value)) if !value.is_empty() => vec![value.clone()],
        _ => Vec::new(),
    }
}

fn required_bool(
    filename: &str,
    map: &BTreeMap<String, FrontValue>,
    key: &str,
) -> Result<bool, ExplainError> {
    match required_string(filename, map, key)?.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(ExplainError::new(format!(
            "{filename}: {key} must be true or false, got {other}"
        ))),
    }
}

fn parse_kind(filename: &str, value: &str) -> Result<Kind, ExplainError> {
    match value {
        "keyword" => Ok(Kind::Keyword),
        "operator" => Ok(Kind::Operator),
        "annotation" => Ok(Kind::Annotation),
        "type" => Ok(Kind::Type),
        "modifier" => Ok(Kind::Modifier),
        "legacy" => Ok(Kind::Legacy),
        "concept" => Ok(Kind::Concept),
        other => Err(ExplainError::new(format!(
            "{filename}: unknown kind {other}"
        ))),
    }
}

fn validate_entry(filename: &str, entry: &Entry) -> Result<(), ExplainError> {
    if entry.syntax.trim().is_empty() {
        return Err(ExplainError::new(format!(
            "{filename}: syntax must not be empty"
        )));
    }
    if entry.summary.ends_with('.') {
        return Ok(());
    }
    Err(ExplainError::new(format!(
        "{filename}: summary must be a sentence ending with '.'"
    )))
}

fn insert_unique(
    map: &mut BTreeMap<String, usize>,
    key: &str,
    index: usize,
    label: &str,
) -> Result<(), ExplainError> {
    if let Some(previous) = map.insert(key.to_owned(), index) {
        return Err(ExplainError::new(format!(
            "duplicate {label} {key} in entries {previous} and {index}"
        )));
    }
    Ok(())
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Keyword => "keyword",
            Kind::Operator => "operator",
            Kind::Annotation => "annotation",
            Kind::Type => "type",
            Kind::Modifier => "modifier",
            Kind::Legacy => "legacy",
            Kind::Concept => "concept",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_entries_are_available() {
        let registry = Registry::load().expect("registry loads");
        assert!(registry.lookup("≡").is_some());
        assert!(registry.lookup("→").is_some());
        assert!(registry.lookup("proba").is_some());
    }

    #[test]
    fn lookup_resolves_terms_aliases_and_legacy() {
        let registry = Registry::load().expect("registry loads");
        assert!(matches!(registry.lookup("≡"), Some(Lookup::Exact(entry)) if entry.term == "≡"));
        assert!(
            matches!(registry.lookup("equals"), Some(Lookup::Alias { entry, .. }) if entry.term == "≡")
        );
        assert!(
            matches!(registry.lookup("=="), Some(Lookup::Legacy { canonical, .. }) if canonical.term == "≡")
        );
    }

    #[test]
    fn all_embedded_entries_validate() {
        let registry = Registry::load().expect("registry loads");
        assert!(registry.entries().len() >= 28);

        for entry in registry.entries() {
            assert!(!entry.term.is_empty());
            assert!(!entry.category.is_empty());
            assert!(!entry.syntax.is_empty());
            assert!(!entry.summary.is_empty());
            assert!(
                first_faber_example(&entry.body).is_some(),
                "{} missing fab example",
                entry.term
            );
        }
    }

    #[test]
    fn unknown_frontmatter_fields_fail() {
        let raw = [RawEntry {
            filename: "bad.md",
            source: r#"---
term: "bad"
kind: "concept"
category: "test"
canonical: true
summary: "Bad entry."
syntax: "bad"
surprise: "no"
---

Body.

```fab
incipit {}
```
"#,
        }];

        let err = Registry::from_raw_entries(&raw).expect_err("unknown field fails");
        assert!(err.message.contains("unknown frontmatter field surprise"));
    }

    #[test]
    fn render_plain_includes_short_contract() {
        let registry = Registry::load().expect("registry loads");
        let lookup = registry.lookup("≡").expect("lookup equality");
        let rendered = render_plain(&lookup);
        assert!(rendered.contains("Kind: operator"));
        assert!(rendered.contains("Syntax: <expression> ≡ <expression>"));
        assert!(rendered.contains("adfirma 1 + 1 ≡ 2"));
    }

    #[test]
    fn render_json_is_valid() {
        let registry = Registry::load().expect("registry loads");
        let lookup = registry.lookup("≡").expect("lookup equality");
        let rendered = render_json(&lookup).expect("json renders");
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("valid json");
        assert_eq!(value["term"], "≡");
        assert_eq!(value["canonical"], true);
    }
}
