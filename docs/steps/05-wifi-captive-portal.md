# Step 5 — Wi-Fi Captive Portal Provisioning

**Status:** implementation complete, untested on hardware (branch `step_05_wifi_captive_portal`).
**Goal:** The device has no hardcoded Wi-Fi credentials. On first boot it becomes an access point named `PoolMon-XXXX`; the user joins it from their phone, is auto-redirected to a config form by the OS's captive-portal detection, enters their home Wi-Fi credentials, submits, and the device saves them to NVS and reboots into station (client) mode on the home network. Every subsequent boot reads the saved credentials and goes straight to STA.

This is the single biggest step in the project so far — **5 new Rust files, 2 updated ones, and one C-config change** — so this doc is long. Skim the "File layout" and "State machine" sections first; the per-file walkthrough is for when you want to understand a specific piece.

## File layout

```
temp_monitor/src/wifi/
├── mod.rs          public API + state machine orchestration (96 lines)
├── credentials.rs  NVS read / write / clear (60 lines)
├── ap.rs           SoftAP bring-up (40 lines)
├── sta.rs          STA (client) connection (30 lines)
├── dns.rs          DNS hijack — UDP server (95 lines)
└── http.rs         HTTP server — form + save handler + redirects (160 lines)
```

Everything else that changed:

- `temp_monitor/src/main.rs` — enable the `Wifi` service.
- `temp_monitor/sdkconfig.defaults` — bumps for HTTP server stack + lwIP sockets.

## State machine

```
                ┌─────────────────────────────┐
                │          boot               │
                └──────────────┬──────────────┘
                               │
                               ▼
            ┌───────────────────────────────┐
            │ Credentials::load(nvs)        │
            └──┬────────────┬───────────────┘
               │            │
         None/Err       Some(creds)
               │            │
               │            ▼
               │     ┌──────────────────────┐
               │     │ sta::connect()       │
               │     └──┬───────────────┬───┘
               │        │               │
               │     Err(…)            Ok
               │        │               │
               │   wipe creds           ▼
               │        │          ┌────────────────────┐
               │        │          │ STA up, got IP     │
               │        │          │ (return to main,   │
               │        │          │  sensor loop runs) │
               │        │          └────────────────────┘
               │        │
               └────────┘
                   │
                   ▼
         ┌──────────────────────┐
         │ ap::start()          │  open AP "PoolMon-XXXX"
         │ DnsServer::start()   │  UDP 53, answers any A query
         │ http::start(nvs)     │  HTTP 80, form + /save
         └──────────┬───────────┘
                    │
                    ▼
         (return to main; sensor loop runs;
          phone-side interaction drives state)
                    │
        user submits form via POST /save
                    │
                    ▼
         ┌──────────────────────┐
         │ Credentials::save()  │
         │ esp_restart()        │
         └──────────┬───────────┘
                    │
                    ▼
              (next boot, STA path)
```

The **main loop keeps running the sensor** while the device is in provisioning mode — the Wi-Fi state machine doesn't block it. The HTTP + DNS servers run in their own tasks (FreeRTOS for HTTP, a Rust thread for DNS), so `main` returns as soon as all three are up.

## `wifi/mod.rs` — public API

```rust
pub struct Wifi {
    _http: Option<EspHttpServer<'static>>,
    _dns: Option<DnsServer>,
    _wifi: BlockingWifi<EspWifi<'static>>,
}
```

The struct owns three long-lived handles. The leading underscores are because they're never *read* — they exist purely to extend the lifetime of the running services. The moment you drop the `Wifi` struct, fields are dropped in declaration order: HTTP server shuts down, DNS thread gets a signal to stop, finally the radio is de-configured.

**`Wifi::start(modem, sys_loop, nvs)`** is the entry point. It tries the STA path first if it can, and falls back to AP/provisioning on failure. Note `modem: Modem<'static>` — explicit because `Modem` is lifetime-parameterised and `EspWifi::new` needs a matching lifetime.

The three arguments are **owned peripherals/handles**:

- `modem`: the Wi-Fi/BLE radio. Singleton — `peripherals.modem`. Ownership means no one else can touch the radio for this program's lifetime.
- `sys_loop`: `EspSystemEventLoop::take()`. Needed for Wi-Fi event dispatch (connected, disconnected, got IP, etc.). `EspWifi` uses it internally.
- `nvs`: `EspDefaultNvsPartition::take()`. Handle to the "nvs" partition (which the ESP-IDF default partition table always has). Both `EspWifi::new` and our `credentials` module use it.

All three are Rust's **peripheral singleton** pattern — `take()` returns `Ok` once, then `Err` forever. The compiler prevents you from re-taking.

## `wifi/credentials.rs` — NVS read/write

Non-volatile storage on the ESP32 is the `nvs` partition (look at the Step 1 boot log: `0 nvs WiFi data 01 02 00009000 00006000` — that's a 24 KB partition at flash offset 0x9000). NVS is a key-value store with namespaces (tables); we use a namespace called `"wifi"` and two keys `"ssid"` and `"pw"`.

```rust
let nvs = EspNvs::<NvsDefault>::new(partition.clone(), "wifi", true)?;
nvs.set_str("ssid", "...")?;                 // write
nvs.get_str("ssid", &mut buf)?               // read → Option<&str>
```

The `true` in `EspNvs::new` is "create namespace if missing". `partition.clone()` is cheap — `EspDefaultNvsPartition` is ref-counted.

Note we don't encrypt at rest. NVS supports encryption but setting that up is its own project and the threat model here (my pool) doesn't warrant it.

**`clear()`** wipes the keys — used when STA fails repeatedly so the next boot goes back to provisioning. Right now it runs once on any STA failure; for a real product you'd retry N times first.

## `wifi/ap.rs` — SoftAP

```rust
let ap_config = AccessPointConfiguration {
    ssid: "PoolMon-AB12".try_into()?,
    auth_method: AuthMethod::None,
    channel: 1,
    max_connections: 4,
    ssid_hidden: false,
    ..Default::default()
};
wifi.set_configuration(&Configuration::AccessPoint(ap_config))?;
wifi.start()?;
wifi.wait_netif_up()?;
```

Three things worth noting:

1. **SSID derived from MAC** (`PoolMon-AB12`). Two boards on the same bench get different names. `wifi.wifi().ap_netif().get_mac()` returns the AP MAC, and we hex-format the last two bytes.
2. **`AuthMethod::None`** — open network. Counter-intuitive ("security!") but correct: the user doesn't know a password yet, that's literally what we're provisioning, so requiring one creates a chicken-and-egg. The AP is only up for a few minutes at first boot and carries one HTTP form — low value target.
3. **`wait_netif_up()`** — blocks until the AP's DHCP is ready to hand out leases. Without this, the HTTP server may bind before the network interface is actually listening.

DHCP is handled entirely by ESP-IDF. The board becomes `192.168.71.1`; connecting phones get `192.168.71.2+`. This subnet is configurable via `CONFIG_LWIP_SOFTAP_*` in sdkconfig.

## `wifi/sta.rs` — STA (client) mode

```rust
let client = ClientConfiguration {
    ssid: creds.ssid.as_str().try_into()?,
    password: creds.password.as_str().try_into()?,
    auth_method: AuthMethod::WPA2Personal,
    ..Default::default()
};
wifi.set_configuration(&Configuration::Client(client))?;
wifi.start()?;
wifi.connect()?;
wifi.wait_netif_up()?;
```

**`AuthMethod::WPA2Personal`** — covers essentially every home Wi-Fi network made in the last 15 years. If the AP is actually WPA3 or the older WPA, ESP-IDF negotiates. The `AuthMethod::None` variant is only for unsecured networks; otherwise WPA2Personal is the sensible default.

`wifi.connect()` starts the association and returns after the STA actually associates — but we also need `wait_netif_up()` because "associated" ≠ "has IP"; DHCP happens after association.

## `wifi/dns.rs` — the captive-portal trick

This is the file worth studying for "how does captive portal actually work?".

Every phone OS probes well-known URLs when joining a new network to decide "am I on the real Internet?":

| OS | Probe URL | Expected response |
|---|---|---|
| iOS / macOS | `http://captive.apple.com/hotspot-detect.html` | `<HTML>...<BODY>Success</BODY>...</HTML>` verbatim |
| Android | `http://connectivitycheck.gstatic.com/generate_204` | HTTP 204 empty |
| Windows | `http://www.msftconnecttest.com/connecttest.txt` | `Microsoft Connect Test` |

If the OS gets the expected response, the network is "real"; the phone shows the usual full-bar icon and does nothing special. If it gets anything else (400, 302, wrong body, connection refused, etc.), the OS flags the network as **captive**, and the corresponding UX kicks in — on iOS/macOS a captive-portal sheet auto-opens in Safari, on Android a notification tells you to sign in.

For those probes to reach us at all, the phone's DNS lookup for `captive.apple.com` has to resolve to **our IP**, not the real one. That's what `dns.rs` does: binds UDP port 53, and answers every A-record query with `192.168.71.1` (us).

**The DNS protocol is tiny** — the whole implementation is 95 lines. A query looks like this:

```
  offset   0  1  2  3  4  5  6  7  8  9 10 11
header:  [ID ][FLG ][QD   ][AN   ][NS   ][AR   ]
  12+:   [name labels ... 0][QTYPE][QCLASS]
```

Labels are length-prefixed bytes (e.g. `5 a p p l e 3 c o m 0` for `apple.com`). The response is the same header (with flags changed to "response, no error"), the original question copied verbatim, then one answer record: `0xC00C` (a compression pointer back to offset 12, "same name as the question"), type A (1), class IN (1), TTL (we use 60s), length 4, and four bytes of IP.

Worth reading the code with [RFC 1035](https://www.rfc-editor.org/rfc/rfc1035) open. The `build_response` function parses enough of the question to find where QTYPE/QCLASS start (so we can echo them back), then slaps together the 12-byte answer record. It doesn't even look at the question name — we don't care; we answer everything identically.

The server runs in a `std::thread`, not a FreeRTOS task directly. On `esp-idf-svc` those are the same thing under the hood — `std::thread::Builder::new().stack_size(4096).spawn(...)` creates a FreeRTOS task with a specific stack size. A 4 KB stack is plenty for this 512-byte-buffer loop. The `AtomicBool` + 500 ms read timeout lets the thread notice when it's asked to stop and exit cleanly.

## `wifi/http.rs` — the form

Three handlers:

- `GET /` → serves a ~1 KB HTML form (ssid input + password input + Save button). Styled well enough that it looks intentional on a phone, no framework.
- `POST /save` → reads the form body, parses `ssid=...&password=...` (our own tiny `parse_form` + `url_decode`), saves to NVS, serves a "rebooting" page, sleeps 2 seconds, calls `esp_restart()`. The sleep is important — without it the response TCP stream gets truncated when the board resets.
- A cluster of `GET` handlers for the OS probe URLs (`/hotspot-detect.html`, `/generate_204`, `/gen_204`, `/connecttest.txt`, `/ncsi.txt`, `/success.txt`) — each returns HTTP 302 redirect to `http://192.168.71.1/`. Many phones respect the redirect and auto-load our form in the captive browser.

Why register specific paths instead of a catch-all? Because `EspHttpServer` in esp-idf-svc doesn't support wildcard routes. If you ever see a phone OS that probes a URL not in this list, you'll see a 404 from esp-http-server's built-in handler — still enough to trigger captive detection, but you won't get a visible auto-open until you add the URL.

**Why `Arc` around NVS?** The `POST /save` handler runs on the HTTP server's task, separate from `main`. The closure captures `nvs`, which has to be `'static` for the lifetime of the server. `EspDefaultNvsPartition` is itself cheaply cloneable (it's a refcount under the hood), so `Arc` isn't strictly necessary here — but the explicit `Arc<EspDefaultNvsPartition>` makes the lifetime relationship clearer.

## `wifi/mod.rs` — orchestration

Pulls the pieces together. The only subtlety is the field ordering on the `Wifi` struct:

```rust
pub struct Wifi {
    _http: Option<EspHttpServer<'static>>,
    _dns: Option<DnsServer>,
    _wifi: BlockingWifi<EspWifi<'static>>,
}
```

Rust drops struct fields in declaration order. HTTP server drops first (stops accepting connections), then the DNS server (thread exits), finally the radio (AP tears down). Reverse order would risk the radio being gone while HTTP handlers are still trying to use its IP.

## `main.rs` — three new handles

```rust
let peripherals = Peripherals::take()?;
let sys_loop = EspSystemEventLoop::take()?;
let nvs = EspDefaultNvsPartition::take()?;
```

`EspSystemEventLoop` is the singleton `esp_event` loop. Every `take()`-able thing returns `Ok` once, `Err` thereafter; the compiler ensures you only claim each one exactly once.

## `sdkconfig.defaults` — why the bumps

- `CONFIG_HTTPD_STACK_SIZE=8192` — the HTTP server task runs our handlers. Our `/save` handler calls `Credentials::save` (NVS writes) and `esp_restart()`, which together have non-trivial stack usage. The default 4 KB occasionally gets close. Cheap insurance.
- `CONFIG_HTTPD_MAX_REQ_HDR_LEN=1024`, `CONFIG_HTTPD_MAX_URI_LEN=512` — modern browsers send long `Accept`, `User-Agent`, `Cookie` headers. Defaults are tight. Widen both.
- `CONFIG_LWIP_MAX_SOCKETS=16` — we host an HTTP listener, a DNS listener, an internal DHCP server (ESP-IDF does this), the connecting phone's sockets, and (later) STA sockets. Default 10 is enough in theory; 16 gives margin.

## What's not done

Genuine gaps that are worth being honest about:

1. **No STA retry logic.** If STA fails we wipe creds immediately. In a real product you'd retry 3–5 times (the home Wi-Fi might just be rebooting) before wiping.
2. **No factory-reset button.** Long-pressing BOOT at startup should wipe creds and force provisioning. Easy addition — Step 7 (button wakeup) touches the same GPIO.
3. **Open provisioning AP.** Anyone within Wi-Fi range in the first few minutes of first boot could submit *their* SSID. Minor concern for a pool monitor but worth knowing. Password-protecting the AP (e.g. with the device's serial number printed on a sticker) fixes this.
4. **No HTTPS on the portal.** Phones won't complain (they know the portal is HTTP on a captive network), but we're sending passwords over HTTP. Self-signed TLS is possible but adds ~60 KB binary and a lot of code.
5. **No scan-and-pick.** The form asks you to type the SSID. `EspWifi::scan()` exists and returns the surrounding networks — feeding them into a `<select>` dropdown is a one-afternoon upgrade.
6. **Not tested on actual hardware.** Build is clean; runtime behaviour has to be verified on the board.

## Testing it

```
cargo run --release
```

Expected first-boot log sequence (no credentials in NVS):

```
I (…) temp_monitor: Wi-Fi: no saved credentials — entering provisioning mode
I (…) temp_monitor: SoftAP up. SSID: PoolMon-XXXX
I (…) temp_monitor: Join the AP and browse to http://192.168.71.1/
I (…) temp_monitor: DNS hijack listening on 0.0.0.0:53 → 192.168.71.1
I (…) temp_monitor: HTTP provisioning server listening on :80
I (…) temp_monitor: Services started. Entering main loop.
I (…) temp_monitor: 8A00001125DC9D28   XX.XX °C  (res Bits12)
```

Phone-side:

1. Open Wi-Fi settings, find `PoolMon-XXXX`, tap connect. Your OS will warn "no internet" — expected.
2. Within 10–30 seconds, the phone should auto-open the captive portal showing our form. If not, open any browser and go to `http://192.168.71.1/` directly.
3. Enter your home Wi-Fi SSID and password. Hit Save.
4. The phone-side page says "Credentials saved — rebooting". The board reboots.
5. On reboot log: `Wi-Fi: found saved credentials for SSID=..., attempting STA` → `Wi-Fi STA connected. SSID=... IP=192.168.X.Y`.

To force re-provisioning: wipe NVS from a host with `espflash erase-partition --partition nvs`, or (in code) call `Credentials::clear(&nvs)`.

## Gotchas during implementation

All captured in [`docs/gotchas.md`](../gotchas.md), but the short list:

- `embedded_svc::...` isn't a direct dep — use `esp_idf_svc::io::Write`, `esp_idf_svc::http::Method`, etc.
- `Modem<'d>` is lifetime-parameterised. `Wifi::start` signature must say `Modem<'static>`.
- `EspNvs::set_str` takes `&self`, not `&mut self` — it's interior-mutable through FreeRTOS locks.
- `EspHttpServer` doesn't support wildcard routes; register the captive-check URLs explicitly.

## Next step

Step 6 — deep sleep between readings. Once the sensor is reading, Wi-Fi is connected, and readings are posting (step 8), deep sleep is what makes the battery last. The Wi-Fi service will need a `stop()` that cleanly disconnects before sleep; right now `Wifi` only drops on program exit.
