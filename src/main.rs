mod decoder;
mod tracker;

use std::{env, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};

use crate::tracker::{TrackerRequest, TrackerResponse};

#[derive(Debug)]
enum Command {
    Decode,
    Info,
    Peers,
}

impl FromStr for Command {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "decode" => Ok(Command::Decode),
            "info" => Ok(Command::Info),
            "peers" => Ok(Command::Peers),
            _ => Err(()),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Torrent {
    announce: String,
    info: TorrentInfo,
}

#[derive(Serialize, Deserialize)]
struct TorrentInfo {
    length: u32,
    name: String,
    #[serde(rename = "piece length")]
    piece_length: usize,
    pieces: ByteBuf,
    // #[serde(flatten)]
    // keys: Keys,
}

// #[derive(Debug, Clone, Deserialize, Serialize)]
// enum Keys {
//     SingleFile { length: usize },
//     MultipleFiles { files: File },
// }

// #[derive(Debug, Clone, Deserialize, Serialize)]
// struct File {
//     legnth: usize,
//     path: Vec<String>,
// }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let command: Result<Command, ()> = args[1].parse();

        match command {
            Ok(cmd) => match cmd {
                // cargo run decode (something)
                Command::Decode => {
                    let bencoded_value = decoder::BencodedValue::decode(&args[2]);
                    println!("Decoded value: {}", bencoded_value.value.to_string());
                    return Ok(());
                }
                // cargo run info sample.torrent
                Command::Info => {
                    let file_path = &args[2];
                    let contents = match std::fs::read(file_path) {
                        Ok(contents) => contents,
                        Err(_) => {
                            eprint!("File does not exist");
                            return Ok(());
                        }
                    };

                    let torrent = serde_bencode::from_bytes::<Torrent>(&contents).unwrap();
                    let bytes = serde_bencode::to_bytes(&torrent.info).unwrap();
                    let hash = hex::encode(Sha1::digest(bytes));

                    // println!("Pieces {:?}", torrent.info.pieces);
                    println!("Tracker URL: {}", torrent.announce);
                    println!("Length: {}", torrent.info.length);
                    println!("Info hash: {}", hash);
                    // println!("Piece Length: {}", torrent.info.piece_length);
                    // println!("Piece hashes:");
                    // for piece in torrent.info.pieces.chunks(20) {
                    //     let hash = hex::encode(piece);
                    //     println!("{hash}");
                    // }
                }
                // cargo run peers sample.torrent
                Command::Peers => {
                    let file_path = &args[2];
                    let contents = match std::fs::read(file_path) {
                        Ok(contents) => contents,
                        Err(_) => {
                            eprintln!("File does not exist");
                            return Ok(());
                        }
                    };

                    let torrent = serde_bencode::from_bytes::<Torrent>(&contents).unwrap();
                    let bytes = serde_bencode::to_bytes(&torrent.info).unwrap();
                    // let hash = hex::encode(Sha1::digest(bytes));

                    let mut hasher = Sha1::new();
                    hasher.update(&bytes);
                    let info_hash = hasher.finalize();

                    println!("Tracker URL: {}", torrent.announce);
                    // println!("Info hash: {}", info_hash);

                    let request = TrackerRequest::new(torrent.info.length as usize);
                    let url_params = serde_urlencoded::to_string(&request).unwrap();
                    let tracker_url = format!(
                        "{}?{}&info_hash={}",
                        torrent.announce,
                        url_params,
                        urlencode(&info_hash.into())
                    );

                    let response = reqwest::get(tracker_url).await?.bytes().await?;
                    let response: TrackerResponse = serde_bencode::from_bytes(&response).unwrap();
                    for peer in response.peers.0 {
                        println!("{}:{}", peer.ip(), peer.port());
                    }
                }
            },
            Err(_) => {
                println!("Invalid command");
            }
        }
    } else {
        eprintln!("..");
    }
    Ok(())
}

fn urlencode(t: &[u8; 20]) -> String {
    let mut encoded = String::with_capacity(3 * t.len());

    for &byte in t {
        encoded.push('%');
        encoded.push_str(&hex::encode(&[byte]));
    }

    encoded
}
