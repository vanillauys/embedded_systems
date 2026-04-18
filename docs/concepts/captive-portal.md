# Captive portal provisioning

The "join a Wi-Fi network, a sign-in page pops up automatically, you enter credentials, it connects you to the real Internet" UX you see at airports, coffee shops, and — now — your pool monitor.

## The four pieces

Every captive portal — commercial or embedded — needs four things running simultaneously on the device hosting the AP:

| Piece | Purpose | Our impl |
|---|---|---|
| **SoftAP** | An advertised Wi-Fi network the client can join. | ESP32 radio in AP mode, `EspWifi` configured with `AccessPointConfiguration` |
| **DHCP server** | Hands joining clients an IP on the AP's subnet. | ESP-IDF runs this automatically in AP mode |
| **DNS server** | Resolves *any* name to our IP, so probe URLs land on us. | [`wifi/dns.rs`](../../temp_monitor/src/wifi/dns.rs), ~95 lines |
| **HTTP server** | Serves the sign-in page and handles the form submit. | [`wifi/http.rs`](../../temp_monitor/src/wifi/http.rs), `EspHttpServer` |

Remove any one and the UX breaks:

- No DNS hijack → the phone's OS successfully resolves `captive.apple.com` to the *real* Apple IP, which it can't reach (we're not the real Internet) → phone gives up and just shows "no internet", never auto-opening the portal.
- No HTTP server → the phone's DNS lookup succeeds (points at us), but the HTTP request gets connection-refused. Some phones handle this gracefully and open the portal; others don't.
- No DHCP → the phone can't even associate cleanly with the AP.
- No SoftAP → you don't have a provisioning surface at all.

## Captive-portal detection, per OS

When a phone joins a new Wi-Fi network, the OS doesn't trust that "associated" means "on the Internet". It sends an HTTP probe to a known-good URL and checks the response. If the response matches what the OS expects, the network is deemed "real". If not, it's "captive" and the portal UX kicks in.

| OS | Probe URL | Expected response | Fall-through behaviour |
|---|---|---|---|
| iOS / macOS | `http://captive.apple.com/hotspot-detect.html` | exactly `<HTML>…<BODY>Success</BODY>…</HTML>` | auto-pops Safari sheet onto the first URL the OS tries |
| Android (AOSP) | `http://connectivitycheck.gstatic.com/generate_204` | HTTP `204 No Content`, empty body | notification + browser page at `/generate_204` redirect target |
| Android (some OEMs) | additional `/gen_204` endpoints | `204 No Content` | same |
| Windows 10/11 | `http://www.msftconnecttest.com/connecttest.txt` | body `Microsoft Connect Test` | notification; browser on click |
| Chrome OS | similar to Android | | |

Our strategy: **respond to all of these with HTTP 302 redirect to `http://192.168.71.1/`**. Most phones treat a redirect as "not the expected success response" → captive detected → they load the redirect target in the portal browser.

One exception worth knowing: **iOS requires exactly the success HTML** at `captive.apple.com/hotspot-detect.html` for the network to be deemed real. If you want to keep iPhones *on your AP* (e.g. briefly after provisioning while you tear down AP and bring up STA), you'd return that exact body. We don't do that — we want iOS to show the portal, so we redirect.

## Why DNS hijack at all? Why not just HTTP?

Without DNS hijack, every probe URL resolves to its real IP. The phone would try to reach `17.x.x.x` (Apple's CDN) over our AP. Since we're not actually internet-connected, the packet dies in our lwIP stack. The phone can't tell if that's "network failure" or "captive portal"; some OSes interpret it one way, some the other, and the UX isn't consistent.

By **hijacking DNS** — answering every query with our own IP — we force every HTTP probe to come to us. Then we unambiguously return a non-success response and the OS commits to captive-portal UX.

It's also what keeps the user from accidentally browsing while on the AP. Try to visit `google.com`? DNS resolves to us. We serve the config form. This makes the portal UX feel like "the network literally is the config page", which is what we want.

## Our minimal DNS server

Full protocol in [RFC 1035](https://www.rfc-editor.org/rfc/rfc1035). We implement the smallest possible subset:

- Listen on UDP port 53.
- For every incoming query, build a response with:
  - Same transaction ID (bytes 0-1).
  - Flags set to "response, authoritative, no error" (`0x8580`).
  - One answer record: name compression pointer to the question's name (`0xC00C`), type A, class IN, TTL 60 s, RDLENGTH 4, 4 bytes of IP.
- Don't validate the query. Don't care what name was asked about. Answer everything with our IP.

That's 95 lines of Rust. If anything else (SOA, AAAA, TXT, DNSSEC, EDNS) came through, we'd mis-answer or drop it, but phones don't care.

One gotcha: IPv6 queries (AAAA) come in alongside A queries on modern phones. We return the *same* response (type A, which is wrong for AAAA). Phones don't seem to mind — they try A next and use that. If we ever wanted to be correct we'd either respond with AAAA answers pointing at an IPv6 ULA, or respond NOERROR with no answer records (RFC 8020) to tell the phone "no AAAA exists, try A".

## When the user submits the form

The `/save` handler is the state-transition point. It:

1. Reads the form body.
2. Writes the credentials to NVS.
3. Serves a "saved, rebooting" page.
4. Sleeps 2 seconds (so the TCP stream actually flushes).
5. Calls `esp_restart()`.

The reboot is a deliberate choice, not laziness. An alternative — "gracefully tear down AP, bring up STA in place" — is possible but annoying:

- `EspWifi` is a singleton; reconfiguring it mid-run works but is fiddly.
- The AP's DHCP server, DNS hijack, and HTTP server all need to be stopped cleanly.
- If the STA config is wrong, you're mid-transition with no AP to fall back to.

Rebooting gives us a clean slate. The next boot reads NVS, tries STA, and if STA fails can fall back to AP the same way. It's the "turn it off and on again" pattern, and it works.

## Further reading

- RFC 7710 — Captive-portal identification via DHCP option 160.
- RFC 8908 — The "Captive Portal API" (JSON document at a well-known URL) — ongoing standardisation to replace ad-hoc probing.
- ESP-IDF captive_portal_example — <https://github.com/espressif/esp-idf/tree/master/examples/protocols/http_server/captive_portal> — the official C reference we mostly mirror.
