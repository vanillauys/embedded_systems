#![allow(dead_code)] // stub module; real implementation lands at step 5.

//! Wi-Fi service — **stub, not yet implemented**.
//!
//! Planned for **step 5** of the learning plan. The chosen approach is to use
//! `esp-idf-svc`'s built-in provisioning workflow (SoftAP flavour): on first
//! boot with no credentials in NVS, the device exposes its own access point
//! running a captive portal; the user connects with a phone, picks their
//! home Wi-Fi, enters the password, and the device persists the credentials
//! and reconnects in station mode on every subsequent boot.
//!
//! Until step 5 is started, this module exists only so `main.rs` can
//! document *where* the service would be enabled and what dependencies it
//! takes (the `Modem` peripheral). Calling `Wifi::start` returns an error.
//!
//! See [`docs/concepts/service-modules.md`] for the design rationale.

use esp_idf_svc::hal::modem::Modem;

/// Handle for the Wi-Fi service. Owns the modem peripheral while alive.
pub struct Wifi;

impl Wifi {
    /// Take the modem peripheral and bring up Wi-Fi. Not yet implemented.
    pub fn start(_modem: Modem) -> anyhow::Result<Self> {
        anyhow::bail!("wifi::Wifi::start not yet implemented — planned for step 5 (esp-idf-svc SoftAP provisioning)")
    }
}
