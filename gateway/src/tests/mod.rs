use std::net::SocketAddr;

pub mod auth;
pub mod lb;

fn create_socket_addr(count: i32) -> Vec<SocketAddr> {
    let mut addrs = Vec::<SocketAddr>::new();

    for i in 0..count {
        addrs.push(format!("127.0.0.{}:3000", i).parse().unwrap());
    }

    addrs
}
