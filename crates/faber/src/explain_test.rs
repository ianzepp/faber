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
        source: r#"+++
term = "bad"
kind = "concept"
category = "test"
canonical = true
summary = "Bad entry."
syntax = "bad"
surprise = "no"
+++

Body.

```fab
incipit {}
```
"#,
    }];

    let err = Registry::from_raw_entries(&raw).expect_err("unknown field fails");
    // TOML + serde(deny_unknown_fields) produces "unknown field `surprise`..."
    assert!(err.message.contains("surprise") && err.message.contains("unknown field"));
}

#[test]
fn old_delimiters_frontmatter_fail() {
    let raw = [crate::explain::RawEntry {
        filename: "old.md",
        source: r#"---
term = "old"
kind = "concept"
category = "test"
canonical = true
summary = "Old entry."
syntax = "old"
+++

Body with fab example.

```fab
incipit {}
```
"#,
    }];

    let err = Registry::from_raw_entries(&raw).expect_err("old --- must fail");
    assert!(
        err.message.contains("missing frontmatter"),
        "old YAML --- should be rejected, got: {}",
        err.message
    );
}

#[test]
fn missing_or_unterminated_toml_frontmatter_fails() {
    let missing = [crate::explain::RawEntry {
        filename: "miss.md",
        source: r#"term = "x"
kind = "concept"
category = "t"
canonical = true
summary = "S."
syntax = "x"

```fab
incipit {}
```
"#,
    }];
    let err = Registry::from_raw_entries(&missing).expect_err("missing +++");
    assert!(err.message.contains("missing frontmatter"));

    let unterm = [crate::explain::RawEntry {
        filename: "unterm.md",
        source: r#"+++
term = "x"
kind = "concept"
category = "t"
canonical = true
summary = "S."
syntax = "x"

Body.

```fab
incipit {}
```
"#,
    }];
    let err = Registry::from_raw_entries(&unterm).expect_err("unterminated");
    assert!(err.message.contains("unterminated frontmatter"));
}

#[test]
fn toml_frontmatter_type_errors_reported() {
    // scalar where array required (aliases must be array in schema)
    let scalar_array = [crate::explain::RawEntry {
        filename: "badarr.md",
        source: r#"+++
term = "x"
kind = "concept"
category = "t"
canonical = true
summary = "S."
syntax = "x"
aliases = "not-an-array"
+++

```fab
incipit {}
```
"#,
    }];
    let err = Registry::from_raw_entries(&scalar_array).expect_err("scalar for array");
    assert!(err.message.contains("aliases") || err.message.contains("invalid type"));

    // non-string item in list
    let bad_item = [crate::explain::RawEntry {
        filename: "baditem.md",
        source: r#"+++
term = "x"
kind = "concept"
category = "t"
canonical = true
summary = "S."
syntax = "x"
examples = [1, 2]
+++

```fab
incipit {}
```
"#,
    }];
    let err = Registry::from_raw_entries(&bad_item).expect_err("non-string array item");
    assert!(err.message.contains("examples") || err.message.contains("invalid type"));
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
    assert!(rendered.contains("not canonical Faber source"));
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
        "demum" => "fac",
        "tempta" => "fac",
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
            "demum" => "demum",
            "tempta" => "tempta",
            other => panic!("unexpected legacy term {other:?}"),
        }
    } else {
        match term {
            "=" => "equals",
            "≡" => "eq-eq",
            "≠" => "bang-eq",
            "≤" => "lt-eq",
            "≥" => "gt-eq",
            "→" => "arrow",
            "⇥" => "exit-arrow",
            "←" => "assign",
            "⊕" => "plus-eq",
            "⊖" => "minus-eq",
            "⊘" => "slash-eq",
            "⊛" => "star-eq",
            "⇢" => "verte",
            "⇒" => "conversio",
            "∧" => "amp",
            "∨" => "pipe",
            "∪" => "cup",
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
            "∴" => "therefore",
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
    // Minimal TOML frontmatter extractor for test helper (only "term" etc. used).
    // Looks for +++ delimited block and simple `key = "value"` (or 'value') lines.
    let mut lines = source.lines();
    if lines.next()? != "+++" {
        return None;
    }

    for line in lines {
        if line.trim() == "+++" {
            break;
        }
        // Match key = "..." or key="..." (common in our corpus)
        let line = line.trim();
        if let Some(eq_pos) = line.find('=') {
            let found_key = line[..eq_pos].trim();
            if found_key != key {
                continue;
            }
            let mut val = line[eq_pos + 1..].trim();
            // strip optional surrounding quotes (single or double)
            if ((val.starts_with('"') && val.ends_with('"'))
                || (val.starts_with('\'') && val.ends_with('\'')))
                && val.len() >= 2
            {
                val = &val[1..val.len() - 1];
            }
            return Some(val.to_owned());
        }
    }

    None
}
