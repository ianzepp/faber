//! `faber explain` — language reference lookup.

use crate::cli::ExplainArgs;
use faber::explain::{self, Registry};

/// Renders explain-corpus entries with CLI policies around mutually exclusive modes.
pub(super) fn cmd_explain(args: ExplainArgs) {
    let registry = match Registry::load() {
        Ok(registry) => registry,
        Err(err) => {
            eprintln!("error: failed to load explain corpus: {err}");
            std::process::exit(1);
        }
    };

    if args.list {
        print!("{}", explain::render_list(&registry));
        return;
    }

    if let Some(category) = args.category {
        match explain::render_category(&registry, &category) {
            Some(output) => print!("{output}"),
            None => {
                eprintln!("error: no explanations found in category '{category}'");
                let categories = registry
                    .categories()
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(", ");
                eprintln!("hint: available categories: {categories}");
                std::process::exit(1);
            }
        }
        return;
    }

    if let Some(query) = args.search {
        if args.json {
            eprintln!("error: --json cannot be combined with --search");
            std::process::exit(2);
        }
        if args.term.is_some() {
            eprintln!("error: --search cannot be combined with a term");
            std::process::exit(2);
        }

        let hits = registry.search(&query);
        if hits.is_empty() {
            eprintln!("error: no matches found for '{query}'");
            eprintln!("hint: run `faber explain --list`");
            std::process::exit(1);
        }

        print!("{}", explain::render_search(&query, &hits));
        return;
    }

    let Some(term) = args.term else {
        eprintln!("error: no explain query was provided");
        eprintln!();
        eprintln!("Usage:");
        eprintln!("    faber explain <term>");
        eprintln!("    faber explain --list");
        eprintln!("    faber explain --category <category>");
        eprintln!("    faber explain --search <query>");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("    faber explain functio");
        eprintln!("    faber explain ≡");
        eprintln!("    faber explain ==");
        eprintln!("    faber explain --list");
        std::process::exit(2);
    };

    let Some(lookup) = registry.lookup(&term) else {
        eprintln!("error: no explanation found for '{term}'");
        eprintln!("hint: run `faber explain --list`");
        std::process::exit(1);
    };

    if args.json {
        match explain::render_json(&lookup) {
            Ok(json) => println!("{json}"),
            Err(err) => {
                eprintln!("error: {err}");
                std::process::exit(1);
            }
        }
    } else {
        print!("{}", explain::render_plain(&lookup));
    }
}
