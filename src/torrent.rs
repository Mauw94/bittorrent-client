use std::{
    fs,
    io::{Read, Write},
    net::{SocketAddrV4, TcpStream},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use crate::{
    peer::{Message, MessageTag},
    tracker::{Peer, TrackerResponse},
};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Torrent {
    pub announce: String,
    pub info: Info,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Info {
    pub name: String,
    pub length: usize,
    #[serde(rename = "piece length")]
    pub piece_length: usize,
    #[serde(with = "serde_bytes")]
    pub pieces: Vec<u8>,
}

const PEER_ID: &[u8; 20] = b"00112233445566778899"; // use this peer_id for the challenge

impl Torrent {
    pub fn new(path: PathBuf) -> Result<Torrent, anyhow::Error> {
        let torrent_byte: Vec<u8> = fs::read(path)?;
        // eprintln!("{:?}", torrent_byte);
        let decoded: Torrent = serde_bencode::from_bytes(&torrent_byte)?;

        Ok(decoded)
    }

    pub fn info_hash_hex(&self) -> Result<String, anyhow::Error> {
        let bytes = serde_bencode::to_bytes(&self.info)?;
        let mut hasher = <Sha1 as Digest>::new();
        hasher.update(bytes);
        let hash = hasher.finalize();
        let hex = hex::encode(hash);

        Ok(hex)
    }

    pub fn info_hash_bytes(&self) -> [u8; 20] {
        let bytes = serde_bencode::to_bytes(&self.info).expect("it must be valid bytes");
        let mut hasher = <Sha1 as Digest>::new();
        hasher.update(bytes);

        hasher.finalize().try_into().expect("array")
    }

    pub fn info_hash_url_encoded(&self) -> Result<String, anyhow::Error> {
        let hash = self.info_hash_hex()?;

        let mut url_encoded = String::new();

        for (i, char) in hash.chars().enumerate() {
            if i % 2 == 0 {
                url_encoded.push('%');
            }
            url_encoded.push(char);
        }

        Ok(url_encoded)
    }

    pub async fn discover_peers(&self) -> Result<Vec<Peer>, anyhow::Error> {
        let endpoint = format!(
            "{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}",
            self.announce,
            self.info_hash_url_encoded().unwrap(),
            "00112233445566778899",
            6881,
            0,
            0,
            self.info.length,
            1
        );

        let response = reqwest::get(endpoint).await?.bytes().await?;
        let decoded: TrackerResponse = serde_bencode::from_bytes(&response)?;

        Ok(decoded.all_peers())
    }

    fn make_handshake(
        &self,
        stream: &mut TcpStream,
        peer_addr: SocketAddrV4,
        peer_id: [u8; 20],
    ) -> anyhow::Result<()> {
        println!("Connected to peer {peer_addr}");
        let mut message = Vec::with_capacity(68);

        message.push(19);

        // string BitTorrent protocol
        for byte in b"BitTorrent protocol" {
            message.push(*byte);
        }

        // eight reserved bytes, which are all set to zero
        for byte in [0u8; 8] {
            message.push(byte);
        }

        // let bytes = serde_bencode::to_bytes(&torrent.info).unwrap();
        for byte in self.info_hash_bytes() {
            message.push(byte);
        }

        // peer id (you can use 00112233445566778899 for this challenge)
        for byte in peer_id {
            message.push(byte);
        }

        eprintln!(
            "
        Peer handhsake sent {:?} of length {}, to {peer_addr}",
            &message,
            message.len()
        );
        stream.write_all(message.as_slice())?;

        Ok(())
    }

    pub async fn peer_handshake(&self, peer_addr: SocketAddrV4) -> anyhow::Result<String> {
        let mut stream = TcpStream::connect(peer_addr)?;
        self.make_handshake(&mut stream, peer_addr, *PEER_ID)?;
        let mut buffer = [0u8; 68];
        stream.read_exact(&mut buffer)?;
        // println!("{:?}", &buffer[48..]);

        Ok(hex::encode(&buffer[48..]))
    }

    pub async fn download_piece(&self, piece_index: u32) -> anyhow::Result<Vec<u8>> {
        let peers = self.discover_peers().await?;
        let peer = peers.first().expect("there is no peer");

        let mut stream = TcpStream::connect(peer.addr())?;

        // make handshake and receive the first message
        self.make_handshake(&mut stream, peer.addr(), *PEER_ID)?;
        let mut buffer = [0u8; 68];
        stream.read_exact(&mut buffer)?;

        Message::read_message(&mut stream); // Read bitfield message

        // send interest message
        let message = Message {
            payload: Vec::new(),
            tag: MessageTag::Interested,
        }
        .as_bytes();
        stream.write_all(&message)?;

        // Wait until we receive unchoke message
        loop {
            let peer_message = Message::read_message(&mut stream); // Read Unchoke message
            if peer_message.tag == MessageTag::Unchoke {
                break;
            }
        }

        let mut piece_data = Vec::new();

        let mut block_index: u32 = 0;
        let mut block_length: u32 = 16 * 1024;

        let mut remaining_bytes = if piece_index < (self.info.pieces.len() / 20 - 1) as u32 {
            // a piece hash is 20 bytes in length
            self.info.piece_length
        } else {
            let last_len = self.info.length % self.info.piece_length;

            if last_len == 0 {
                self.info.piece_length
            } else {
                last_len
            }
        };

        while remaining_bytes != 0 {
            eprintln!(
                "1: {}, {}, {}, {}, {}, {}",
                remaining_bytes,
                piece_index,
                block_index,
                block_length,
                self.info.pieces.len(),
                self.info.piece_length
            );
            if remaining_bytes < block_length as usize {
                block_length = remaining_bytes as u32;
            }

            // send request message
            Message::send_request_piece(
                &mut stream,
                piece_index as u32,
                block_index * (16 * 1024),
                block_length,
            );

            let read_incoming_message = Message::read_message(&mut stream);
            eprintln!("02: {:?}", read_incoming_message.tag);
            if read_incoming_message.tag == MessageTag::Piece {
                eprintln!("2");
                piece_data.extend(read_incoming_message.payload);
            }
            remaining_bytes -= block_length as usize;
            block_index += 1;

            eprintln!(
                "3: {}, {}, {}, {}",
                remaining_bytes, piece_index, block_index, block_length
            );
        }

        Ok(piece_data)
    }
}
