//! Wi-Fi service — captive-portal provisioning + STA (client) connection.
//!
//! ## State machine
//!
//! ```text
//!     boot
//!       │
//!       ▼
//!   load credentials from NVS
//!       │
//!       ├──── none ─────┐
//!       │               │
//!       ▼               │
//!   try STA connect     │
//!       │               │
//!       ├─── fail ──────┤
//!       │               │
//!       └─── ok ───▶ listen forever (sensor loop runs, Wi-Fi stays up)
//!                       │
//!                       ▼
//!                 AP + DNS hijack + HTTP form
//!                       │
//!                       ▼
//!                 user submits creds
//!                       │
//!                       ▼
//!                   save → reboot
//!                       │
//!                       ▼
//!                  (new boot, goes STA)
//! ```
//!
//! ## Modules
//!
//! - [`credentials`] — NVS read/write.
//! - [`ap`] — SoftAP bring-up (open network, SSID derived from MAC).
//! - [`sta`] — STA (client) mode connect using saved credentials.
//! - [`dns`] — UDP DNS server that answers every A query with our IP.
//! - [`http`] — HTTP server serving the form + save/reboot handler.
//!
//! See `docs/steps/05-wifi-captive-portal.md` for the full walkthrough.

mod ap;
mod credentials;
mod dns;
mod http;
mod sta;

use std::net::Ipv4Addr;

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::http::server::EspHttpServer;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};

use credentials::Credentials;
use dns::DnsServer;

/// ESP-IDF's default AP IP. DHCP on the AP hands out 192.168.71.2+. If you
/// change this (`CONFIG_LWIP_SOFTAP_LOCAL_IP`), update this constant too.
const AP_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 71, 1);

/// Wi-Fi service handle. Owns the radio configuration for the lifetime of
/// the program. If the device is in provisioning mode, also owns the
/// running HTTP server and DNS server handles.
pub struct Wifi {
    // These fields keep the radio, HTTP server, and DNS server alive.
    // Dropping `Wifi` tears them all down in field declaration order.
    _http: Option<EspHttpServer<'static>>,
    _dns: Option<DnsServer>,
    _wifi: BlockingWifi<EspWifi<'static>>,
}

impl Wifi {
    /// Entry point for the Wi-Fi service.
    ///
    /// - If credentials are saved in NVS, attempts STA connect. On success
    ///   returns with STA active; the main loop can proceed. On failure the
    ///   credentials are wiped and we fall through to provisioning mode.
    /// - If credentials are absent (or STA failed), brings up SoftAP with
    ///   the DNS hijack + HTTP server and returns. `main` keeps running
    ///   the sensor loop while the phone-side provisioning happens; on
    ///   successful provisioning the device reboots (from inside the HTTP
    ///   `/save` handler) and the next boot takes the STA path.
    pub fn start(
        modem: Modem<'static>,
        sys_loop: EspSystemEventLoop,
        nvs: EspDefaultNvsPartition,
    ) -> anyhow::Result<Self> {
        let mut wifi = BlockingWifi::wrap(
            EspWifi::new(modem, sys_loop.clone(), Some(nvs.clone()))?,
            sys_loop,
        )?;

        match Credentials::load(&nvs)? {
            Some(creds) => {
                log::info!("Wi-Fi: found saved credentials for SSID={:?}, attempting STA", creds.ssid);
                match sta::connect(&mut wifi, &creds) {
                    Ok(()) => Ok(Self {
                        _http: None,
                        _dns: None,
                        _wifi: wifi,
                    }),
                    Err(e) => {
                        log::warn!("Wi-Fi: STA connect failed: {e:#}");
                        log::warn!("Wi-Fi: wiping saved credentials and falling back to provisioning");
                        let _ = Credentials::clear(&nvs);
                        Self::enter_provisioning(wifi, nvs)
                    }
                }
            }
            None => {
                log::info!("Wi-Fi: no saved credentials — entering provisioning mode");
                Self::enter_provisioning(wifi, nvs)
            }
        }
    }

    /// Bring up AP + DNS + HTTP and return with them held. The phone-side
    /// interaction is what drives the state forward from here — the
    /// HTTP `/save` handler triggers a reboot.
    fn enter_provisioning(
        mut wifi: BlockingWifi<EspWifi<'static>>,
        nvs: EspDefaultNvsPartition,
    ) -> anyhow::Result<Self> {
        ap::start(&mut wifi)?;
        let dns = DnsServer::start(AP_IP)?;
        let http = http::start(nvs)?;

        Ok(Self {
            _http: Some(http),
            _dns: Some(dns),
            _wifi: wifi,
        })
    }
}
