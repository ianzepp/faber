use crate::explain::{render_json, render_plain, render_search, Lookup, Registry, RAW_ENTRIES};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CoverageManifest {
    canonical_terms: Vec<String>,
    legacy_terms: Vec<String>,
    excluded_terms: Vec<String>,
}

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
    assert!(
        matches!(registry.lookup("==="), Some(Lookup::Legacy { canonical, .. }) if canonical.term == "est")
    );
    assert!(
        matches!(registry.lookup("!=="), Some(Lookup::Legacy { canonical, .. }) if canonical.term == "non est")
    );
}

#[test]
fn coverage_manifest_matches_registry() {
    let manifest = load_manifest();
    let registry = Registry::load().expect("registry loads");

    assert_eq!(
        registry.entries().len(),
        manifest.canonical_terms.len() + manifest.legacy_terms.len(),
        "registry entry count should match the manifest"
    );

    for term in &manifest.canonical_terms {
        assert_canonical_term(&registry, term);
        assert_embedded_filename(term, false);
    }

    for term in &manifest.legacy_terms {
        let canonical_term = expected_legacy_canonical(term);
        assert_legacy_term(&registry, term, canonical_term);
        assert_embedded_filename(term, true);
    }

    for term in &manifest.excluded_terms {
        assert!(
            registry.lookup(term).is_none(),
            "excluded term {term:?} unexpectedly has explain coverage"
        );
    }
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
            entry.body.contains("```fab"),
            "{} missing fab example",
            entry.term
        );
    }
}

#[test]
fn unknown_frontmatter_fields_fail() {
    let raw = [crate::explain::RawEntry {
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
    assert!(rendered.contains("NAME"));
    assert!(rendered.contains("SYNTAX"));
    assert!(rendered.contains("DESCRIPTION"));
    assert!(rendered.contains("<expression> ≡ <expression>"));
    assert!(rendered.contains("operator / comparison"));
    assert!(rendered.contains("adfirma 1 + 1 ≡ 2"));
    assert!(rendered.contains("RELATED"));
}

#[test]
fn render_legacy_uses_distinct_layout() {
    let registry = Registry::load().expect("registry loads");
    let lookup = registry.lookup("===").expect("lookup legacy equality");
    let rendered = render_plain(&lookup);
    assert!(rendered.contains("STATUS"));
    assert!(rendered.contains("legacy"));
    assert!(rendered.contains("USE INSTEAD"));
    assert!(rendered.contains("est"));
    assert!(rendered.contains("Legacy equality spelling"));
}

#[test]
fn search_returns_ranked_candidates() {
    let registry = Registry::load().expect("registry loads");
    let hits = registry.search("equality");
    assert!(!hits.is_empty());
    assert_eq!(hits[0].entry.term, "≡");
    assert!(hits.iter().any(|hit| hit.entry.term == "est"));

    let rendered = render_search("equality", &hits);
    assert!(rendered.starts_with("Search: equality"));
    assert!(rendered.contains("≡"));
    assert!(rendered.contains("est"));
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

fn load_manifest() -> CoverageManifest {
    toml::from_str(include_str!("../../../explain/coverage.toml")).expect("coverage manifest")
}

fn assert_canonical_term(registry: &Registry, term: &str) {
    match registry.lookup(term) {
        Some(Lookup::Exact(entry)) => assert_eq!(entry.term, term),
        other => panic!("expected canonical lookup for {term:?}, got {other:?}"),
    }
}

fn assert_legacy_term(registry: &Registry, term: &str, canonical_term: &str) {
    match registry.lookup(term) {
        Some(Lookup::Legacy {
            entry, canonical, ..
        }) => {
            assert_eq!(entry.term, term);
            assert!(!entry.canonical);
            assert_eq!(canonical.term, canonical_term);
        }
        other => panic!("expected legacy lookup for {term:?}, got {other:?}"),
    }
}

fn assert_embedded_filename(term: &str, legacy: bool) {
    let expected_filename = expected_filename(term, legacy);
    let raw = RAW_ENTRIES
        .iter()
        .find(|raw| frontmatter_value(raw.source, "term").as_deref() == Some(term))
        .unwrap_or_else(|| panic!("missing embedded file for {term:?}"));
    assert_eq!(
        raw.filename, expected_filename,
        "filename policy mismatch for {term:?}"
    );
}

fn expected_legacy_canonical(term: &str) -> &'static str {
    match term {
        "==" => "≡",
        "!=" => "≠",
        "<=" => "≤",
        ">=" => "≥",
        "->" => "→",
        "===" => "est",
        "!==" => "non est",
        other => panic!("unexpected legacy term {other:?}"),
    }
}

fn expected_filename(term: &str, legacy: bool) -> String {
    let slug = if legacy {
        match term {
            "==" => "eq-eq",
            "!=" => "bang-eq",
            "<=" => "lt-eq",
            ">=" => "gt-eq",
            "->" => "arrow",
            "===" => "est",
            "!==" => "non-est",
            other => panic!("unexpected legacy term {other:?}"),
        }
    } else {
        match term {
            "≡" => "eq-eq",
            "≠" => "bang-eq",
            "≤" => "lt-eq",
            "≥" => "gt-eq",
            "→" => "arrow",
            "←" => "assign",
            "⊕" => "plus-eq",
            "⊖" => "minus-eq",
            "⊘" => "slash-eq",
            "⊛" => "star-eq",
            "⇢" => "verte",
            "⇒" => "conversio",
            "∧" => "amp",
            "∨" => "pipe",
            "⊻" => "caret",
            "¬" => "tilde",
            "≪" => "sinistratum",
            "≫" => "dextratum",
            "‥" => "dot-dot",
            "…" => "ellipsis",
            "?." => "question-dot",
            "?[" => "question-bracket",
            "?(" => "question-paren",
            "!." => "bang-dot",
            "![" => "bang-bracket",
            "!(" => "bang-paren",
            "⊜" => "amp-eq",
            "⊚" => "pipe-eq",
            "@" => "at",
            "§" => "section",
            "non est" => "non-est",
            "solum_in" => "solum-in",
            other => other,
        }
    };

    if legacy {
        format!("{slug}.legacy.md")
    } else {
        format!("{slug}.md")
    }
}

fn frontmatter_value(source: &str, key: &str) -> Option<String> {
    let mut lines = source.lines();
    if lines.next()? != "---" {
        return None;
    }

    for line in lines {
        if line == "---" {
            break;
        }
        let Some((found_key, raw_value)) = line.split_once(':') else {
            continue;
        };
        if found_key.trim() != key {
            continue;
        }

        let value = raw_value.trim();
        if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
            return Some(value[1..value.len() - 1].to_owned());
        }
        return Some(value.to_owned());
    }

    None
}
