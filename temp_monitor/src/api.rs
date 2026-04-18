#![allow(dead_code)] // stub module; real implementation lands at step 8.

//! HTTP(S) API client — **stub, not yet implemented**.
//!
//! Planned for **step 8** of the learning plan. The chosen approach is to
//! use `esp_idf_svc::http::client::EspHttpConnection` for TLS, serialise
//! `sensor::Reading` into JSON, and POST to a backend. First milestone will
//! target [webhook.site](https://webhook.site) for end-to-end smoke testing;
//! a real backend comes later.
//!
//! Until step 8 is started, `post_reading` returns an error. The struct
//! exists so `main.rs` can document how the service slots in.
//!
//! See [`docs/concepts/service-modules.md`] for the design rationale.

use crate::sensor::Reading;

/// Handle for the API client service.
pub struct ApiClient {
    base_url: String,
}

impl ApiClient {
    /// Construct a client targeting `base_url`. Does not open any connections
    /// until the first `post_reading` call.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    /// POST a reading to the backend. Not yet implemented.
    pub fn post_reading(&self, _r: &Reading) -> anyhow::Result<()> {
        anyhow::bail!("api::ApiClient::post_reading not yet implemented — planned for step 8 (HTTPS via esp_idf_svc::http::client::EspHttpConnection)")
    }
}
