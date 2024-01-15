use std::{
    fs,
    io::{Read, Write},
    net::{SocketAddrV4, TcpStream},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use crate::tracker::{Peer, TrackerResponse};

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

impl Torrent {
    pub fn new(path: PathBuf) -> Result<Torrent, anyhow::Error> {
        let torrent_byte: Vec<u8> = fs::read(path)?;
        eprintln!("{:?}", torrent_byte);
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

        hasher.finalize().try_into().expect("GenericArray")
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
        let decoed: TrackerResponse = serde_bencode::from_bytes(&response)?;

        Ok(decoed.all_peers())
    }

    pub async fn peer_handshake(&self, peer_addr: SocketAddrV4) -> anyhow::Result<String> {
        let mut stream = TcpStream::connect(peer_addr)?;

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
        for byte in b"00112233445566778899" {
            message.push(*byte);
        }

        eprintln!(
            "
        Sent {:?} of length {}, to {peer_addr}",
            &message,
            message.len()
        );
        stream.write_all(message.as_slice())?;

        let mut buffer = [0u8; 68];
        stream.read_exact(&mut buffer)?;

        Ok(hex::encode(&buffer[48..]))
    }
}
