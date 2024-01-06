mod decoder;

use std::{env, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};

#[derive(Debug)]
enum Command {
    Decode,
    Info,
}

impl FromStr for Command {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "decode" => Ok(Command::Decode),
            "info" => Ok(Command::Info),
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
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let command: Result<Command, ()> = args[1].parse();

        match command {
            Ok(cmd) => match cmd {
                // cargo run decode (something)
                Command::Decode => {
                    let bencoded_value = decoder::BencodedValue::decode(&args[2]);
                    println!("Decoded value: {}", bencoded_value.value.to_string());
                }
                // cargo run info sample.torrent
                Command::Info => {
                    let file_path = &args[2];
                    let contents = match std::fs::read(file_path) {
                        Ok(contents) => contents,
                        Err(_) => {
                            eprint!("File does not exist");
                            return;
                        }
                    };

                    let torrent = serde_bencode::from_bytes::<Torrent>(&contents).unwrap();
                    let bytes = serde_bencode::to_bytes(&torrent.info).unwrap();
                    let hash = hex::encode(Sha1::digest(bytes));

                    println!("Pieces {:?}", torrent.info.pieces);
                    println!("Tracker URL: {}", torrent.announce);
                    println!("Length: {}", torrent.info.length);
                    println!("Info hash: {}", hash);
                    println!("Piece Length: {}", torrent.info.piece_length);
                    println!("Piece hashes:");
                    for piece in torrent.info.pieces.chunks(20) {
                        let hash = hex::encode(piece);
                        println!("{hash}");
                    }
                }
            },
            Err(_) => println!("Invalid command"),
        }
    }
}
