//! Tiny DNS hijack server for captive-portal detection.
//!
//! When a phone joins our SoftAP, its OS probes "am I actually on the real
//! Internet?" by asking DNS for known probe URLs (e.g.
//! `connectivitycheck.gstatic.com` on Android, `captive.apple.com` on iOS).
//! We answer *every* DNS query with our own IP (192.168.71.1). The phone
//! then HTTP-GETs the probe URL, hits our HTTP server, receives something
//! other than the expected "success" response, decides the network is
//! captive, and auto-opens its captive-portal browser — which lands on our
//! config form.
//!
//! DNS protocol reminder:
//!   Header: 12 bytes (ID, flags, 4× count fields)
//!   Question: name (length-prefixed labels, 0-terminated) + QTYPE + QCLASS
//!   Answer: pointer or name + TYPE + CLASS + TTL + RDLENGTH + RDATA
//!
//! See RFC 1035. We only need to answer A-record queries; everything else
//! we respond to with NXDOMAIN (phones still accept that as "not real
//! Internet").

use std::net::{Ipv4Addr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

/// Background handle for the running DNS server. Dropping it stops the
/// server (the thread exits at its next packet or when the socket is
/// dropped).
pub struct DnsServer {
    _handle: JoinHandle<()>,
    running: Arc<AtomicBool>,
}

impl DnsServer {
    /// Bind UDP port 53 on `0.0.0.0` and spawn a thread that answers every
    /// A query with `ip`.
    pub fn start(ip: Ipv4Addr) -> anyhow::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:53")?;
        // Short read timeout so the loop can notice when we're asked to stop.
        socket.set_read_timeout(Some(std::time::Duration::from_millis(500)))?;

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let handle = thread::Builder::new()
            .name("dns-hijack".into())
            .stack_size(4096)
            .spawn(move || {
                log::info!("DNS hijack listening on 0.0.0.0:53 → {ip}");
                let mut buf = [0u8; 512];
                while running_clone.load(Ordering::Relaxed) {
                    match socket.recv_from(&mut buf) {
                        Ok((len, src)) => {
                            if let Some(resp) = build_response(&buf[..len], ip) {
                                let _ = socket.send_to(&resp, src);
                            }
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock
                            || e.kind() == std::io::ErrorKind::TimedOut => {
                            // Read timeout — loop and recheck the stop flag.
                        }
                        Err(e) => log::warn!("DNS recv: {e}"),
                    }
                }
                log::info!("DNS hijack stopped");
            })?;

        Ok(Self {
            _handle: handle,
            running,
        })
    }
}

impl Drop for DnsServer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

/// Build a DNS response pointing the queried name at `ip`.
///
/// Returns `None` if the incoming packet is malformed — we just ignore it.
fn build_response(query: &[u8], ip: Ipv4Addr) -> Option<Vec<u8>> {
    // Minimum: 12-byte header + at least one byte of question name + QTYPE
    // + QCLASS = 12 + 1 + 4 = 17.
    if query.len() < 17 {
        return None;
    }

    // Parse the question name to figure out where the QTYPE/QCLASS sit and
    // echo them back. Find the terminator (0) in the name field.
    let mut i = 12;
    while i < query.len() && query[i] != 0 {
        // Label = length byte + that many bytes.
        i += 1 + (query[i] as usize);
        if i >= query.len() {
            return None;
        }
    }
    // `query[i]` is the terminator. QTYPE and QCLASS follow — 4 bytes total.
    let question_end = i + 1 + 4;
    if question_end > query.len() {
        return None;
    }

    let mut out = Vec::with_capacity(question_end + 16);
    // Copy the 2-byte transaction ID verbatim.
    out.extend_from_slice(&query[..2]);
    // Flags: QR=1, OPCODE=0, AA=1, TC=0, RD=1, RA=1, Z=0, RCODE=0 → 0x8580
    // (AA=authoritative, RA=recursion-available makes phones happier).
    out.extend_from_slice(&[0x85, 0x80]);
    // QDCOUNT=1, ANCOUNT=1, NSCOUNT=0, ARCOUNT=0.
    out.extend_from_slice(&[0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]);
    // The original question, verbatim.
    out.extend_from_slice(&query[12..question_end]);
    // Answer section:
    //   Name: 0xC00C = message-compression pointer back to offset 12, i.e.
    //     "the name we just parsed in the question".
    //   TYPE=A (0x0001), CLASS=IN (0x0001)
    //   TTL=60 seconds (brief, so mistakes can be corrected fast)
    //   RDLENGTH=4
    //   RDATA=<ip octets>
    out.extend_from_slice(&[0xC0, 0x0C, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3C, 0x00, 0x04]);
    out.extend_from_slice(&ip.octets());

    Some(out)
}
