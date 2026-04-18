//! STA (Station / client) mode — connect to a saved home network.
//!
//! Called at boot if we found credentials in NVS. Blocking: we wait until
//! the DHCP lease is obtained (`wait_netif_up`) before returning. If the
//! connection fails (wrong password, network gone, etc.), the error bubbles
//! up and the caller falls back to AP + portal mode.

use esp_idf_svc::wifi::{
    AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi,
};

use super::credentials::Credentials;

/// Try to associate and get an IP. Blocks until connected (or errors).
pub fn connect(wifi: &mut BlockingWifi<EspWifi<'static>>, creds: &Credentials) -> anyhow::Result<()> {
    let client_config = ClientConfiguration {
        ssid: creds.ssid.as_str().try_into()
            .map_err(|_| anyhow::anyhow!("saved SSID is too long"))?,
        password: creds.password.as_str().try_into()
            .map_err(|_| anyhow::anyhow!("saved password is too long"))?,
        // WPA2Personal is the most common home-network variant. ESP-IDF will
        // negotiate down to WPA if the AP only supports that.
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    };

    wifi.set_configuration(&Configuration::Client(client_config))?;
    wifi.start()?;
    wifi.connect()?;
    wifi.wait_netif_up()?;

    let ip = wifi.wifi().sta_netif().get_ip_info()?;
    log::info!("Wi-Fi STA connected. SSID={} IP={}", creds.ssid, ip.ip);

    Ok(())
}
