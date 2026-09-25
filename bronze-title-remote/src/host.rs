use std::net::IpAddr;

use crate::RemoteError;

pub const OLLAMA_DEFAULT_BASE: &str = "http://127.0.0.1:11434";

pub fn is_loopback_host(host: &str) -> bool {
    let host = strip_brackets(host);
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    match host.parse::<IpAddr>() {
        Ok(ip) => ip.is_loopback(),
        Err(_) => false,
    }
}

pub fn hosted_host_blocked(host: &str) -> bool {
    let host = strip_brackets(host);
    if host.eq_ignore_ascii_case("localhost") || host.ends_with(".localhost") {
        return true;
    }
    match host.parse::<IpAddr>() {
        Ok(ip) => ip_blocked(ip),
        Err(_) => false,
    }
}

fn ip_blocked(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || is_unique_local_v6(&v6)
                || is_link_local_v6(&v6)
                || v6
                    .to_ipv4_mapped()
                    .is_some_and(|v4| ip_blocked(IpAddr::V4(v4)))
        }
    }
}

fn is_unique_local_v6(ip: &std::net::Ipv6Addr) -> bool {
    let octets = ip.octets();
    octets[0] == 0xfc || octets[0] == 0xfd
}

fn is_link_local_v6(ip: &std::net::Ipv6Addr) -> bool {
    let octets = ip.octets();
    octets[0] == 0xfe && (octets[1] & 0xc0) == 0x80
}

fn strip_brackets(host: &str) -> &str {
    host.trim().trim_start_matches('[').trim_end_matches(']')
}

pub fn resolve_ollama_base(ollama_host: Option<&str>) -> String {
    let Some(raw) = ollama_host.map(str::trim).filter(|s| !s.is_empty()) else {
        return OLLAMA_DEFAULT_BASE.into();
    };
    match parse_ollama_host(raw) {
        Ok(base) => base,
        Err(()) => OLLAMA_DEFAULT_BASE.into(),
    }
}

fn parse_ollama_host(raw: &str) -> Result<String, ()> {
    let url = if raw.contains("://") {
        raw.to_string()
    } else {
        format!("http://{raw}")
    };
    let rest = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .ok_or(())?;
    let hostport = rest.split('/').next().unwrap_or(rest);
    let host = hostport
        .rsplit_once(':')
        .map(|(h, _)| h)
        .unwrap_or(hostport);
    if !is_loopback_host(host) {
        return Err(());
    }
    Ok(format!("http://{hostport}"))
}

pub fn parse_hosted_base(raw: &str) -> Result<(String, String), RemoteError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(RemoteError::InvalidBase);
    }
    let rest = raw
        .strip_prefix("https://")
        .ok_or(RemoteError::InvalidBase)?;
    if rest.contains("://") || rest.contains('@') {
        return Err(RemoteError::InvalidBase);
    }
    let hostport = rest.split('/').next().unwrap_or(rest);
    if hostport.is_empty() {
        return Err(RemoteError::InvalidBase);
    }
    let host = hostport
        .rsplit_once(':')
        .map(|(h, port)| {
            if port.bytes().all(|b| b.is_ascii_digit()) {
                h
            } else {
                hostport
            }
        })
        .unwrap_or(hostport);
    if hosted_host_blocked(host) {
        return Err(RemoteError::BlockedHost);
    }
    let base = raw.trim_end_matches('/').to_string();
    Ok((base, host.to_string()))
}

#[cfg(test)]
mod host_tests {
    use super::*;

    #[test]
    fn ollama_rejects_non_loopback_and_ignores_bad_env() {
        assert_eq!(
            resolve_ollama_base(Some("http://8.8.8.8:11434")),
            OLLAMA_DEFAULT_BASE
        );
        assert_eq!(
            resolve_ollama_base(Some("example.com:11434")),
            OLLAMA_DEFAULT_BASE
        );
        assert_eq!(
            resolve_ollama_base(Some("127.0.0.1:11434")),
            "http://127.0.0.1:11434"
        );
        assert_eq!(
            resolve_ollama_base(Some("http://localhost:11434")),
            "http://localhost:11434"
        );
        assert_eq!(
            resolve_ollama_base(Some("http://[::1]:11434")),
            "http://[::1]:11434"
        );
        assert!(is_loopback_host("127.0.0.1"));
        assert!(is_loopback_host("::1"));
        assert!(!is_loopback_host("1.1.1.1"));
    }

    #[test]
    fn hosted_refuses_loopback_and_private() {
        assert_eq!(
            parse_hosted_base("http://api.openai.com"),
            Err(RemoteError::InvalidBase)
        );
        assert_eq!(
            parse_hosted_base("https://127.0.0.1/v1"),
            Err(RemoteError::BlockedHost)
        );
        assert_eq!(
            parse_hosted_base("https://localhost"),
            Err(RemoteError::BlockedHost)
        );
        assert_eq!(
            parse_hosted_base("https://192.168.1.9"),
            Err(RemoteError::BlockedHost)
        );
        assert_eq!(
            parse_hosted_base("https://10.0.0.2"),
            Err(RemoteError::BlockedHost)
        );
        assert_eq!(
            parse_hosted_base("https://169.254.1.1"),
            Err(RemoteError::BlockedHost)
        );
        let (base, host) = parse_hosted_base("https://api.openai.com/v1/").expect("openai");
        assert_eq!(base, "https://api.openai.com/v1");
        assert_eq!(host, "api.openai.com");
    }
}
