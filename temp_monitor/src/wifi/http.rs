//! HTTP provisioning server — the "captive portal" page.
//!
//! Listens on TCP 80. Serves:
//!
//!   GET /          → the provisioning form (HTML, embedded below).
//!   POST /save     → reads the form body, parses `ssid=…&password=…`,
//!                    persists them to NVS, serves a "rebooting" page,
//!                    then calls `esp_restart()` after a short delay.
//!   everything else → 302 redirect to `/`.
//!
//! The 302 on unknown paths is what makes the captive-portal auto-open
//! reliably: when the phone probes Apple/Google captivity URLs our DNS
//! server directs the HTTP request at us, we redirect to `/`, and most
//! OSes interpret that as "captive network, show the portal".

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::Write;
use esp_idf_svc::nvs::EspDefaultNvsPartition;

use super::credentials::Credentials;

/// Minimal mobile-friendly HTML form. Intentionally ~1 KB and no external
/// assets, so it loads from flash on any phone.
const FORM_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>Pool Monitor Setup</title>
  <style>
    body { font-family: -apple-system, system-ui, sans-serif; max-width: 420px; margin: 2em auto; padding: 0 1em; color: #222; }
    h1 { font-size: 1.4em; margin-bottom: 0.2em; }
    p { color: #555; }
    label { display: block; margin-top: 1em; font-weight: 500; }
    input { width: 100%; padding: 0.6em; margin-top: 0.3em; border: 1px solid #ccc; border-radius: 4px; box-sizing: border-box; font-size: 1em; }
    button { width: 100%; padding: 0.8em; margin-top: 1.5em; font-size: 1em; background: #0a84ff; color: #fff; border: none; border-radius: 4px; cursor: pointer; }
    button:active { background: #0060d0; }
  </style>
</head>
<body>
  <h1>Pool Monitor Setup</h1>
  <p>Enter the Wi-Fi credentials this device should use.</p>
  <form method="POST" action="/save">
    <label>Network name (SSID)<input type="text" name="ssid" required maxlength="32"></label>
    <label>Password<input type="password" name="password" maxlength="64"></label>
    <button type="submit">Save and reboot</button>
  </form>
</body>
</html>"##;

const SAVED_HTML: &str = r##"<!DOCTYPE html><html><head><meta charset="utf-8"><title>Saved</title>
<style>body{font-family:sans-serif;max-width:420px;margin:2em auto;padding:0 1em;text-align:center}</style>
</head><body><h1>Credentials saved</h1><p>The device is rebooting. You can now disconnect from <strong>PoolMon-*</strong> and wait for it to appear on your Wi-Fi.</p></body></html>"##;

/// Start the HTTP server. Keep the returned handle alive for as long as you
/// want the server running — dropping it shuts down the listening task.
pub fn start(nvs: EspDefaultNvsPartition) -> anyhow::Result<EspHttpServer<'static>> {
    let mut server = EspHttpServer::new(&Configuration::default())?;

    let nvs_for_save = Arc::new(nvs);

    // GET / → the form.
    server.fn_handler("/", Method::Get, |req| -> Result<(), anyhow::Error> {
        let mut resp = req.into_ok_response()?;
        resp.write_all(FORM_HTML.as_bytes())?;
        Ok(())
    })?;

    // POST /save → persist creds, reboot.
    {
        let nvs = nvs_for_save.clone();
        server.fn_handler("/save", Method::Post, move |mut req| -> Result<(), anyhow::Error> {
            let content_len = req
                .header("content-length")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(0);

            // 1 KB is plenty for ssid=…&password=… (both bounded at 32 + 64).
            let mut body = vec![0u8; content_len.min(1024)];
            let mut read_total = 0;
            while read_total < body.len() {
                let n = req.read(&mut body[read_total..])?;
                if n == 0 { break; }
                read_total += n;
            }
            body.truncate(read_total);

            let body_str = std::str::from_utf8(&body).map_err(|e| anyhow::anyhow!("body: {e}"))?;
            let (ssid, password) = parse_form(body_str)?;

            log::info!("Provisioning: saving credentials for SSID={ssid:?}");
            Credentials::save(&nvs, &ssid, &password)?;

            let mut resp = req.into_ok_response()?;
            resp.write_all(SAVED_HTML.as_bytes())?;
            drop(resp);

            // Give the TCP stack a moment to flush the response before we
            // pull the rug.
            thread::sleep(Duration::from_secs(2));
            log::info!("Restarting...");
            unsafe { esp_idf_svc::sys::esp_restart() };
        })?;
    }

    // Everything else → redirect to `/`. This is what makes captive-portal
    // detection fire on most phones: Apple's `captive.apple.com`, Google's
    // `connectivitycheck.gstatic.com/generate_204`, and so on all resolve
    // (thanks to our DNS hijack) to us, hit a URL we haven't registered,
    // fall through to this catch-all, and get a 302 to `/`.
    server.fn_handler("/hotspot-detect.html", Method::Get, redirect_to_root)?;
    server.fn_handler("/generate_204", Method::Get, redirect_to_root)?;
    server.fn_handler("/gen_204", Method::Get, redirect_to_root)?;
    server.fn_handler("/connecttest.txt", Method::Get, redirect_to_root)?;
    server.fn_handler("/ncsi.txt", Method::Get, redirect_to_root)?;
    server.fn_handler("/success.txt", Method::Get, redirect_to_root)?;

    log::info!("HTTP provisioning server listening on :80");
    Ok(server)
}

fn redirect_to_root(
    req: esp_idf_svc::http::server::Request<&mut esp_idf_svc::http::server::EspHttpConnection<'_>>,
) -> Result<(), anyhow::Error> {
    let headers = [("Location", "http://192.168.71.1/")];
    let _resp = req.into_response(302, Some("Found"), &headers)?;
    Ok(())
}

/// `application/x-www-form-urlencoded` parser for our two fields. Tiny and
/// opinionated — returns (ssid, password).
fn parse_form(body: &str) -> anyhow::Result<(String, String)> {
    let mut ssid = None;
    let mut password = String::new();
    for pair in body.split('&') {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        let decoded = url_decode(v);
        match k {
            "ssid" => ssid = Some(decoded),
            "password" => password = decoded,
            _ => {}
        }
    }
    let ssid = ssid.ok_or_else(|| anyhow::anyhow!("missing ssid"))?;
    if ssid.is_empty() {
        anyhow::bail!("ssid is empty");
    }
    Ok((ssid, password))
}

/// Percent-decoder + `+` → space. Enough for the ssid/password fields.
fn url_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => { out.push(' '); i += 1; }
            b'%' if i + 2 < bytes.len() => {
                if let Ok(byte) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                    out.push(byte as char);
                    i += 3;
                } else {
                    out.push(bytes[i] as char);
                    i += 1;
                }
            }
            b => { out.push(b as char); i += 1; }
        }
    }
    out
}
