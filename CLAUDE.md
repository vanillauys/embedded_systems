# Project: Pool Temperature Monitor (ESP32-S3, Rust)

Personal learning project. Not a work repo. Scope: firmware only — no backend, no frontend, no cloud.

See [docs/initial_plan.md](docs/initial_plan.md) for the 10-step learning plan and [docs/progress.md](docs/progress.md) for current state.

## Who Wihan is here

Senior Java/React engineer (11+ years pro). **Rust is new. Embedded is new.** Hardware soldering/wiring is new.

Implications:
- Software engineering fundamentals (types, ownership semantics, async vs blocking, memory, threading) translate — frame Rust concepts against Java analogues when useful ("think of `Result<T,E>` like a checked exception you can't forget to handle", "`Arc<Mutex<T>>` is roughly `AtomicReference`").
- Don't over-explain general programming concepts. Do explain: Rust borrow checker, lifetimes, `unsafe`, macros, trait bounds vs Java generics, and embedded-specific things (memory-mapped IO, interrupts, 1-Wire protocol, RTC memory, deep sleep, voltage levels, open-drain).
- He'll notice and appreciate idiomatic Rust. Don't write Java-in-Rust.

## Project layout

```
embedded_systems/
├── CLAUDE.md                 # this file
├── temp_monitor/             # the Cargo project (from esp-idf-template)
│   ├── Cargo.toml
│   ├── src/main.rs
│   └── ...
└── docs/
    ├── README.md             # index
    ├── progress.md           # current state + checklist — update when finishing a step
    ├── initial_plan.md       # the 10-step plan
    ├── commands.md           # build/flash/monitor cheat sheet
    ├── gotchas.md            # append-only issue log, newest at top
    ├── nix.md                # NixOS env notes
    ├── steps/NN-name.md      # one per learning step — filled in as we go
    ├── concepts/*.md         # Rust / embedded concepts as they come up
    └── hardware/*.md         # pinouts, wiring
```

## How to collaborate here

### Teaching mode
This is a learning project. When walking through a step:
1. State the concrete next command(s) to run, as commands he types himself — don't do everything autonomously.
2. Briefly explain *what's about to happen* and *what to watch for*.
3. After he reports the result, explain what was learned. Flag anything idiomatic about the Rust. Flag any embedded gotchas.
4. When a Rust concept comes up for the first time (ownership, lifetimes, `?`, traits, `unsafe`, macros, `Pin`, async), explain it inline briefly AND suggest adding a file under `docs/concepts/` if it's meaty.

### Documenting as we go
Whenever a step is completed or meaningful progress is made:
- Update `docs/progress.md` — move the checkbox, update "current state", add a history row.
- Write / update the matching `docs/steps/NN-*.md` — what was built, code snippets, surprises, what was learned.
- If a problem was hit: append an entry to `docs/gotchas.md` (newest at top, template at bottom of the file).
- If a new Rust or embedded concept came up meaningfully: add a file under `docs/concepts/`.

Keep docs terse but readable cold. These are notes for future-Wihan who may come back in 3 months.

### Tone
- Concise. Code + commands over prose. He reads fast.
- OK to be opinionated — flag when a choice in the plan is suboptimal.
- Never pad with "great question" / restate-the-request fluff.

## Tooling specific to this project

- **Cargo project lives in `temp_monitor/`** — run `cargo` commands from inside it, not repo root.
- **Toolchain:** Xtensa (`esp` channel via `espup`), pinned via `rust-toolchain.toml`. Don't suggest standard `rustup` toolchains here — they don't produce Xtensa binaries.
- **Flash:** `cargo run --release` works because the template wires up `.cargo/config.toml` with an `espflash flash --monitor` runner.
- **Build times:** first build of ESP-IDF is 5–10 min. Don't suggest `cargo clean` casually — it nukes that work.
- **NixOS:** may need `pkg-config`, `python3`, `cmake`, `ninja`, `llvmPackages.libclang`, `openssl` in the dev shell. See [docs/nix.md](docs/nix.md).

## What NOT to do

- Don't add features, crates, or abstractions beyond what the current step needs. The plan is explicit about what each step introduces — respect the ordering.
- Don't generate diagrams or markdown files unprompted. `docs/` structure is set; just fill it.
- Don't recommend moving to `no_std` or `esp-hal` (the alternative, bare-metal Rust stack). We're on `esp-idf-svc` + `esp-idf-hal` deliberately — gives us Wi-Fi, NVS, OTA, deep sleep for free.
- Don't use jcodemunch to index this repo unless asked. It's a single small Cargo project; Read/Grep/Glob are fine.
