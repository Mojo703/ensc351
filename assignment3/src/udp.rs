use std::{
    io::ErrorKind,
    net::{SocketAddr, UdpSocket},
    sync::Arc,
};

use crate::command;

pub struct UdpConn {
    socket: UdpSocket,
}

impl UdpConn {
    pub fn bind(addr: &str) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        Ok(UdpConn { socket })
    }

    /// Try to receive a command. Returns Ok(None) if no data currently.
    pub fn try_recv_command(&self) -> std::io::Result<Option<(command::Command, SocketAddr)>> {
        let mut buf = [0u8; 1024];
        match self.socket.recv_from(&mut buf) {
            Ok((n, addr)) => {
                if n == 0 {
                    return Ok(None);
                }
                if let Ok(s) = std::str::from_utf8(&buf[..n]) {
                    match s.trim().parse::<command::Command>() {
                        Ok(cmd) => Ok(Some((cmd, addr))),
                        Err(_) => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Send a raw UTF-8 reply to the given address
    pub fn send_reply(&self, reply: Arc<str>, dest: SocketAddr) -> std::io::Result<usize> {
        self.socket.send_to(reply.as_bytes(), dest)
    }
}
