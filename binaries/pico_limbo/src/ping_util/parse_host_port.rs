use std::io;
use std::net::{SocketAddr, ToSocketAddrs};

const DEFAULT_PORT: u16 = 25565;

pub fn parse_host_port(input: &str) -> (String, u16) {
    let default_port = DEFAULT_PORT;

    // Check for an IPv6 literal, which should be wrapped in brackets.
    if input.starts_with('[') {
        // Look for the pattern "]:"
        if let Some(end_bracket) = input.find(']') {
            return if input.len() > end_bracket + 1
                && &input[end_bracket + 1..end_bracket + 2] == ":"
            {
                // Port is provided after the closing bracket.
                let port_str = &input[end_bracket + 2..];
                let port = port_str.parse().unwrap_or(default_port);
                (input[..=end_bracket].to_string(), port)
            } else {
                // No port provided; return the full hostname with default port.
                (input.to_string(), default_port)
            };
        }
    }

    // For non-IPv6 or IPv4/domain names, look for the last colon.
    if let Some(idx) = input.rfind(':') {
        let (host, port_part) = input.split_at(idx);
        // port_part starts with ':', so we trim it.
        let port_str = &port_part[1..];
        if port_str.is_empty() {
            // If a colon is present but no port is specified, use the default.
            return (host.to_string(), default_port);
        }
        // Parse the port; if parsing fails, default is used.
        let port = port_str.parse().unwrap_or(default_port);
        return (host.to_string(), port);
    }

    // No colon found at all, so use the entire input as hostname and default port.
    (input.to_string(), default_port)
}

pub fn parse_socket_addr(addr: &str) -> io::Result<SocketAddr> {
    let addr_with_port = if addr.contains(':') {
        addr.to_string()
    } else {
        format!("{addr}:{DEFAULT_PORT}")
    };

    let mut addrs_iter = addr_with_port.to_socket_addrs()?;
    addrs_iter
        .next()
        .ok_or_else(|| io::Error::other("No valid socket addresses found"))
}

#[cfg(test)]
mod tests {
    use super::parse_host_port;

    #[test]
    fn test_domain_with_port() {
        let input = "example.com:25565";
        let expected = ("example.com".to_string(), 25565);
        assert_eq!(parse_host_port(input), expected);
    }

    #[test]
    fn test_ipv4_with_port() {
        let input = "127.0.0.1:25565";
        let expected = ("127.0.0.1".to_string(), 25565);
        assert_eq!(parse_host_port(input), expected);
    }

    #[test]
    fn test_domain_without_port() {
        let input = "something.org";
        let expected = ("something.org".to_string(), 25565);
        assert_eq!(parse_host_port(input), expected);
    }

    #[test]
    fn test_ipv6_with_port() {
        let input = "[::1]:4321";
        let expected = ("[::1]".to_string(), 4321);
        assert_eq!(parse_host_port(input), expected);
    }

    #[test]
    fn test_ipv6_without_port() {
        let input = "[::1]";
        let expected = ("[::1]".to_string(), 25565);
        assert_eq!(parse_host_port(input), expected);
    }

    #[test]
    fn test_colon_without_port() {
        let input = "example.com:";
        let expected = ("example.com".to_string(), 25565);
        assert_eq!(parse_host_port(input), expected);
    }

    #[test]
    fn test_invalid_port() {
        // When the port is non-numeric, the default port should be used.
        let input = "example.com:abc";
        let expected = ("example.com".to_string(), 25565);
        assert_eq!(parse_host_port(input), expected);
    }
}
