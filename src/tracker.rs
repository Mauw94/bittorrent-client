use std::net::{Ipv4Addr, SocketAddrV4};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerRequest {
    // info_hash: Vec<u8>,
    peer_id: String,
    port: u16,
    uploaded: usize,
    downloaded: usize,
    left: usize,
    compact: bool,
}

impl TrackerRequest {
    #[allow(dead_code)]
    pub fn new(left: usize) -> Self {
        Self {
            peer_id: "00112233445566778899".to_string(),
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            left,
            compact: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Peer(SocketAddrV4);

impl Peer {
    #[allow(dead_code)]
    pub fn addr(&self) -> SocketAddrV4 {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerResponse {
    // pub failure_reason: String,
    pub interval: usize,
    #[serde(with = "serde_bytes")]
    pub peers: Vec<u8>,
}

impl TrackerResponse {
    pub fn all_peers(&self) -> Vec<Peer> {
        let mut peers = Vec::new();
        for chunk in self.peers.chunks(6) {
            let addr = Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
            let port = u16::from_be_bytes([chunk[4], chunk[5]]);
            peers.push(Peer(SocketAddrV4::new(addr, port)));
        }

        peers
    }
}
