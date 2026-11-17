//! UDP listener for LAN room announcements.

use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};

pub const LAN_DISCOVERY_PORT: u16 = 27315;

/// Binds the LAN discovery port with `SO_REUSEADDR`, so several clients on
/// the same machine can all receive the broadcast announcements (a plain bind
/// is exclusive: the second client would never see any LAN room).
pub fn bind_lan_listener(port: u16) -> Option<UdpSocket> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).ok()?;
    socket.set_reuse_address(true).ok()?;
    socket.set_broadcast(true).ok()?;
    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
    socket.bind(&addr.into()).ok()?;
    socket.set_nonblocking(true).ok()?;
    Some(socket.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_listeners_can_share_the_discovery_port() {
        // A free port, so the test never collides with a running game.
        let port = UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();

        let first = bind_lan_listener(port);
        let second = bind_lan_listener(port);

        assert!(first.is_some());
        assert!(second.is_some());
    }
}
