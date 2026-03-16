use super::protocol::Protocol;

/// Parses a server address string into (connect_addr, hostname, protocol).
///
/// Accepts formats like:
/// - `https://domain.com` → (`domain.com:443`, `domain.com`, HTTPS)
/// - `http://domain.com:8080` → (`domain.com:8080`, `domain.com`, HTTP)
/// - `domain.com:3000` → (`domain.com:3000`, `domain.com`, HTTP)
pub fn parse_address(address: &str) -> (String, String, Protocol) {
    let (protocol, rest) =
        if let Some(stripped) = address.strip_prefix("https://") {
            (Protocol::HTTPS, stripped)
        } else if let Some(stripped) = address.strip_prefix("http://") {
            (Protocol::HTTP, stripped)
        } else {
            (Protocol::HTTP, address)
        };

    let default_port = match protocol {
        Protocol::HTTPS => 443,
        Protocol::HTTP => 80,
    };

    let (hostname, connect_addr) =
        if let Some((host, port)) = rest.rsplit_once(':') {
            (host.to_string(), format!("{host}:{port}"))
        } else {
            (rest.to_string(), format!("{rest}:{default_port}"))
        };

    (connect_addr, hostname, protocol)
}
