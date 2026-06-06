use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct ServerAddress {
    host: String,
    port: u16,
}

const DEFAULT_PORT: u16 = 25565;

impl Default for ServerAddress {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: DEFAULT_PORT,
        }
    }
}

impl Display for ServerAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse server address")]
pub struct ParseServerAddressError;

impl ServerAddress {
    /// Parses the bind string into a host and an optional port
    pub fn parse(bind: &str) -> Result<Self, ParseServerAddressError> {
        if bind.is_empty() {
            return Err(ParseServerAddressError);
        }

        // IPv6 addresses are enclosed in square brackets, e.g. [::1]:25565
        if bind.starts_with('[')
            && let Some(end) = bind.find(']')
        {
            let host = bind[1..end].to_string();
            let port = bind[end + 1..]
                .strip_prefix(':')
                .and_then(|p| p.parse().ok())
                .unwrap_or(DEFAULT_PORT);
            if host.is_empty() {
                return Err(ParseServerAddressError);
            }
            return Ok(Self { host, port });
        }

        // IPv4 addresses are separated by a colon, e.g. 127.0.0.1:25565
        if let Some((host, port_str)) = bind.rsplit_once(':')
            && let Ok(port) = port_str.parse::<u16>()
            && !host.contains(':')
        {
            return Ok(Self {
                host: host.to_string(),
                port,
            });
        }

        Ok(Self {
            host: bind.to_string(),
            port: DEFAULT_PORT,
        })
    }

    /// Overrides the port if the provided Option is Some
    pub const fn set_port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }

    /// Converts the struct into the tuple format Tokio expects
    pub fn tuple(&self) -> (&str, u16) {
        (&self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_v4_with_port() {
        let addr = ServerAddress::parse("127.0.0.1:25565").unwrap();
        assert_eq!(addr.host, "127.0.0.1");
        assert_eq!(addr.port, 25565);
    }

    #[test]
    fn test_ip_v4_without_port() {
        let addr = ServerAddress::parse("127.0.0.1").unwrap();
        assert_eq!(addr.host, "127.0.0.1");
        assert_eq!(addr.port, 25565);
    }

    #[test]
    fn test_ip_v6_with_port() {
        let addr = ServerAddress::parse("[::1]:25565").unwrap();
        assert_eq!(addr.host, "::1");
        assert_eq!(addr.port, 25565);
    }

    #[test]
    fn test_ip_v6_without_port() {
        let addr = ServerAddress::parse("[::1]").unwrap();
        assert_eq!(addr.host, "::1");
        assert_eq!(addr.port, 25565);
    }

    #[test]
    fn test_hostname_with_port() {
        let addr = ServerAddress::parse("localhost:25565").unwrap();
        assert_eq!(addr.host, "localhost");
        assert_eq!(addr.port, 25565);
    }

    #[test]
    fn test_hostname_without_port() {
        let addr = ServerAddress::parse("localhost").unwrap();
        assert_eq!(addr.host, "localhost");
        assert_eq!(addr.port, 25565);
    }

    #[test]
    fn test_empty_string() {
        let addr = ServerAddress::parse("");
        assert!(addr.is_err());
    }

    #[test]
    fn test_set_port_overrides_existing_port() {
        let mut addr = ServerAddress::parse("127.0.0.1:30066").unwrap();
        let addr = addr.set_port(30067);
        assert_eq!(addr.host, "127.0.0.1");
        assert_eq!(addr.port, 30067);
    }
}
