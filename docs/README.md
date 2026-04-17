# Pool Monitor — Docs

Personal learning journey: ESP32-S3 + Rust + embedded, building a battery-powered pool temperature monitor.

## Where to find what

| File / dir | Purpose |
|---|---|
| [initial_plan.md](initial_plan.md) | The 10-step learning plan. The source of truth for scope and ordering. |
| [progress.md](progress.md) | Running log of what's done, what's in progress, and the next concrete action. |
| [commands.md](commands.md) | Cheat sheet for `cargo build`, `espflash`, `cargo run`, device listing, etc. |
| [gotchas.md](gotchas.md) | Running list of problems hit and how they were solved. Grep this first when stuck. |
| [nix.md](nix.md) | NixOS-specific environment setup notes. |
| [steps/](steps/) | One markdown file per learning step. Written as the step is completed — what was built, what was learned, surprises. |
| [concepts/](concepts/) | Standalone notes on Rust or embedded concepts that come up. E.g. ownership, peripherals singleton, 1-Wire, deep sleep memory. |
| [hardware/](hardware/) | Pinouts, wiring diagrams, component datasheets/notes. |

## Conventions

- **Step files** (`steps/NN-name.md`) follow a consistent template: what we built, code snippets, what was learned, next step. Future-you should be able to reread one and re-do the step.
- **Concept files** stay focused on one idea. Link them from step files rather than duplicating.
- **Gotchas** are append-only — new entries at the top with a date.
- **Progress** is the single source of truth for "where am I?". Always update it when finishing a step.
