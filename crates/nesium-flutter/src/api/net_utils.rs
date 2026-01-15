/// Get all local IPv4 and IPv6 addresses.
pub fn get_local_addrs() -> Vec<String> {
    if_addrs::get_if_addrs()
        .map(|ifaces| {
            ifaces
                .into_iter()
                .filter(|iface| !iface.is_loopback())
                .map(|iface| iface.ip().to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Try to get the public IP address using an HTTP service.
pub fn get_public_ip() -> Option<String> {
    // Try multiple services for robustness
    let services = [
        "https://api.ipify.org",
        "https://icanhazip.com",
        "https://ident.me",
    ];

    for service in services {
        if let Ok(resp) = ureq::get(service).call() {
            if let Ok(ip_str) = resp.into_string() {
                let trimmed = ip_str.trim();
                // Validate it's a valid IP
                if trimmed.parse::<std::net::IpAddr>().is_ok() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }

    None
}

/// Try to map a port using UPnP.
/// TODO: Implement actual UPnP port mapping.
pub fn try_upnp_mapping(_port: u16, _label_prefix: &str) {
    // use easy_upnp::add_port;
    // let label = format!("{}-{}", _label_prefix, _port);
    // Map TCP
    // let _ = add_port(_port, easy_upnp::Protocol::Tcp, &label);
    // Map UDP (for QUIC)
    // let _ = add_port(_port, easy_upnp::Protocol::Udp, &label);
}
