//! SoftAP bring-up.
//!
//! Puts the Wi-Fi radio in AccessPoint mode with an SSID derived from the
//! board's factory MAC address (last 2 bytes → 4 hex digits). Open network
//! — no password — because the user needs to join it without already knowing
//! a secret, and the only traffic over it is the provisioning form.
//!
//! The ESP-IDF Wi-Fi stack boots a DHCP server automatically in AP mode, so
//! phones that join get `192.168.71.x` (ESP default) with us at `.1`.

use esp_idf_svc::wifi::{AccessPointConfiguration, AuthMethod, BlockingWifi, Configuration, EspWifi};

/// Configure the Wi-Fi radio as an open SoftAP and start it.
///
/// Returns the AP SSID that was set (so `main` can log it / show it in a
/// "how to provision" message).
pub fn start(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<String> {
    let ssid = ap_ssid(wifi)?;

    let ap_config = AccessPointConfiguration {
        ssid: ssid.as_str().try_into()
            .map_err(|_| anyhow::anyhow!("SSID too long for AP config"))?,
        auth_method: AuthMethod::None,
        ssid_hidden: false,
        channel: 1,
        max_connections: 4,
        ..Default::default()
    };

    wifi.set_configuration(&Configuration::AccessPoint(ap_config))?;
    wifi.start()?;
    wifi.wait_netif_up()?;

    log::info!("SoftAP up. SSID: {ssid}");
    log::info!("Join the AP and browse to http://192.168.71.1/");

    Ok(ssid)
}

/// Derive a stable, human-friendly SSID from the board's AP MAC address.
/// `PoolMon-XXXX` where `XXXX` is the last 2 bytes (4 hex chars).
fn ap_ssid(wifi: &BlockingWifi<EspWifi<'static>>) -> anyhow::Result<String> {
    let mac = wifi.wifi().ap_netif().get_mac()?;
    Ok(format!("PoolMon-{:02X}{:02X}", mac[4], mac[5]))
}
