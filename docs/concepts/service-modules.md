# Service modules: the shape of `main.rs`

Rust in general and `esp-idf-svc` in particular don't impose a style for firmware. But we want this project to scale past one `main.rs` without painting ourselves into corners as we add Wi-Fi, HTTPS, persistent config, and deep sleep. This doc captures the pattern we picked and why.

## The goal

A `main` function that reads like a Java main class — one where each subsystem ("service") is visibly started, and the main loop is a thin driver that calls into those services. Something like:

```rust
fn main() -> anyhow::Result<()> {
    init();

    // === Services — (un)comment to enable / disable ================
    let mut sensor = sensor::TemperatureSensor::new(pins.gpio4)?;
    // let _wifi = wifi::Wifi::start(modem)?;
    // let api   = api::ApiClient::new("https://…");

    loop {
        // drive each enabled service once
    }
}
```

The intent is that reading `main.rs` tells you the full inventory of the firmware at a glance.

## The three services

| Service | Owns | Depends on | Status |
|---|---|---|---|
| [`sensor`](../../temp_monitor/src/sensor.rs) | one GPIO pin configured open-drain, a `OneWire` bus, a delay source | `one-wire-bus`, `ds18b20` | working — step 3 |
| [`wifi`](../../temp_monitor/src/wifi.rs) | the `Modem` peripheral (and eventually NVS creds) | `esp-idf-svc` Wi-Fi provisioning | stub — step 5 |
| [`api`](../../temp_monitor/src/api.rs) | a base URL (no network resources — the wifi service owns those) | `esp_idf_svc::http::client::EspHttpConnection` | stub — step 8 |

Each service:
- Is its own Rust module in `src/`.
- Exposes **one struct** as its public API.
- Construction takes ownership of the hardware resources it needs (a GPIO pin, the modem peripheral, etc.). Rust's ownership model then guarantees those resources aren't double-used anywhere else in the firmware.
- Stubs `bail!` on start/action rather than silently returning "OK" — you can't accidentally run with a half-configured service.

## Why structure it this way

### 1. Ownership is the feature, not a chore.

`Peripherals::take()` hands you a struct of every peripheral exactly once. If `sensor::TemperatureSensor::new(pin)` takes ownership of GPIO 4, then nothing else can use GPIO 4 — the compiler won't let you. No runtime "oops, two drivers tried to bit-bang the same pin". This is why the service constructors take the concrete pin/modem/etc. directly rather than an index or a name.

Java analogy: like passing a `Connection` into a repository constructor. Here the "connection" is a piece of silicon.

### 2. Main stays flat.

Everything a reader needs to understand the firmware's shape is visible in `main.rs`. There is no framework, no `#[derive(Service)]`, no central registry. Just plain module calls. This is deliberate — for a 10-step learning project, clarity wins over cleverness.

### 3. Generics are contained inside services.

1-Wire forced some generic gymnastics (`Ds18b20::new`'s phantom error type, the `embedded-hal 0.2` vs `1.0` split, pin-type bounds). `TemperatureSensor` hides all of that behind a non-generic `.read_all()` returning `anyhow::Result<Vec<Reading>>`. When future-Wihan reads `main.rs`, he doesn't have to remember which trait bounds fly.

### 4. Stubs are real modules.

`wifi::Wifi` and `api::ApiClient` are not placeholder comments — they're compiling modules whose sole job right now is to document *where* the service fits and what it *will* own. That means when Step 5 lands, you extend an existing module instead of inventing one.

## Enabling / disabling a service

Two coordinated changes in `main.rs`:

1. (Un)comment the `let x = …::start/new(…)` line in the Services block.
2. (Un)comment any calls to `x.…` inside the main loop.

They're paired so Rust's "unused variable" warning reminds you if you only flipped one of them.

We could hide this behind a config struct, feature flags, or runtime booleans. None of those would be clearer than commenting two lines. Revisit if the firmware ever has more than ~5 services.

## What this is not

- **Not an async runtime.** Main is a blocking loop. `FreeRtos::delay_ms` yields the single FreeRTOS task. If a service needs to run concurrently (e.g. handling HTTP requests during captive-portal provisioning) we'll spawn a FreeRTOS task inside that service, not globally. Embassy / tokio are off the table at this scale.
- **Not a dependency-injection container.** Services know their own dependencies statically; `main` just calls constructors. No registry, no lookup by type.
- **Not a trait-based abstraction over services.** There is no `trait Service { fn start(); fn tick(); }`. Every service has a different shape (some are handles, some are synchronous-only, some need network state). Forcing them through a trait would be over-abstraction.
