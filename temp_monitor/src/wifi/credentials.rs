//! NVS-backed storage for the saved Wi-Fi credentials.
//!
//! Non-volatile storage on the ESP32 is the ESP-IDF `nvs` partition
//! (configured in the partition table; the default table always has one).
//! We take a namespace — a per-app key-value table inside that partition —
//! and store two keys: `ssid` and `pw`.
//!
//! `load` returns `Ok(None)` specifically when there are no credentials
//! saved yet (first boot). Any actual NVS error bubbles up as `Err`.

use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};

const NAMESPACE: &str = "wifi";
const KEY_SSID: &str = "ssid";
const KEY_PW: &str = "pw";

/// Plain-text Wi-Fi credentials. No encryption at rest — NVS is in flash
/// and anyone with the board can dump it. Fine for our threat model.
pub struct Credentials {
    pub ssid: String,
    pub password: String,
}

impl Credentials {
    /// Load credentials from NVS. Returns `Ok(None)` if the SSID key isn't
    /// present, meaning the device is unprovisioned.
    pub fn load(partition: &EspDefaultNvsPartition) -> anyhow::Result<Option<Self>> {
        let nvs = EspNvs::<NvsDefault>::new(partition.clone(), NAMESPACE, true)?;

        let mut ssid_buf = [0u8; 64];
        let ssid = match nvs.get_str(KEY_SSID, &mut ssid_buf)? {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => return Ok(None),
        };

        let mut pw_buf = [0u8; 128];
        let password = nvs
            .get_str(KEY_PW, &mut pw_buf)?
            .map(|s| s.to_string())
            .unwrap_or_default();

        Ok(Some(Self { ssid, password }))
    }

    /// Persist credentials. Overwrites anything currently there.
    pub fn save(partition: &EspDefaultNvsPartition, ssid: &str, password: &str) -> anyhow::Result<()> {
        let nvs = EspNvs::<NvsDefault>::new(partition.clone(), NAMESPACE, true)?;
        nvs.set_str(KEY_SSID, ssid)?;
        nvs.set_str(KEY_PW, password)?;
        Ok(())
    }

    /// Wipe saved credentials. Used when a STA connection attempt fails
    /// repeatedly, to force the device back into provisioning mode.
    #[allow(dead_code)]
    pub fn clear(partition: &EspDefaultNvsPartition) -> anyhow::Result<()> {
        let nvs = EspNvs::<NvsDefault>::new(partition.clone(), NAMESPACE, true)?;
        let _ = nvs.remove(KEY_SSID);
        let _ = nvs.remove(KEY_PW);
        Ok(())
    }
}
