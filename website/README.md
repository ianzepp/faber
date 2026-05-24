# website/

Static documentation site source for the Faber Romanus project.

This directory contains the presentation assets and content sources for the public Faber website (GitHub Pages).

## Monorepo Rationale

The website was originally a sibling repository (`faber-www`). It has been migrated here so that:

- Documentation stays in sync with the exact compiler source, EBNF, explain corpus, examples, and stdlib at every commit.
- Changes to language behavior, CLI, or targets are reviewed and shipped together with their docs.
- No fragile sibling-path sync scripts or drift between "the code" and "the published docs".

## Layout

```
website/
├── content/                 # Markdown sources (TOML frontmatter with +++ delimiters + body; matches explain/ corpus convention)
│   ├── index.md             # Homepage
│   ├── docs/                # Language and tool reference
│   └── ...
├── templates/
│   └── layout.html          # Simple static HTML chrome (title, nav, content injection)
├── styles/
│   └── main.css             # Responsive docs styling (mobile hamburger nav, etc.)
├── docs/
│   └── faber-website-refresh-plan.md   # Historical migration checklist (this task)
└── dist/                    # Generated output (gitignored; produced by site generator)
```

## Building & Serving (Future)

Once the repo-local generator is implemented (see `faber-website-refresh-plan.md` Phase 2):

```bash
# Planned
cargo run -p faber -- website build   # or similar xtask / dedicated bin
# or
faber site build
```

Until then, the static assets here serve as the single source of truth for the visual layer and curated content. Legacy pre-migration content lives under `content/legacy-from-faber-www/`.

## Published Artifacts

When built, the generator also produces:

- `llms.txt` — LLM-friendly site index
- `faber-complete.md` — concatenated full documentation

These are linked from the layout for easy consumption by tools and models.

## Contributing

Edit content under `website/content/`, the template, or styles. Run the (future) build and verify locally before PR.

The site intentionally stays simple and static — no JS frameworks, minimal dependencies in the generator.

See the root `AGENTS.md` and `website/docs/faber-website-refresh-plan.md` for conventions and the detailed migration status.
