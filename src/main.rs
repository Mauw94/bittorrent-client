mod decoder;
mod torrent;
mod tracker;
use clap::{Parser, Subcommand};
use std::{net::SocketAddrV4, path::PathBuf};
use torrent::Torrent;

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Decode {
        encoded_bencode: String,
    },
    Info {
        torrent: PathBuf,
    },
    Peers {
        torrent: PathBuf,
    },
    Handshake {
        torrent: PathBuf,
        peer_addr: SocketAddrV4,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        // cargo run decode (something)
        Commands::Decode { encoded_bencode } => {
            let bencoded_value = decoder::BencodedValue::decode(&encoded_bencode);
            println!("Decoded value: {}", bencoded_value.value.to_string());
            return Ok(());
        }
        // cargo run info sample.torrent
        Commands::Info { torrent } => {
            let torrent = Torrent::new(torrent)?;
            println!("Tracker URL: {}", torrent.announce);
            println!("Length: {}", torrent.info.length);
            println!("Info Hash: {}", torrent.info_hash_hex()?);
            println!("Piece Length: {}", torrent.info.piece_length);
        }
        // cargo run peers sample.torrent
        Commands::Peers { torrent } => {
            let torrent = Torrent::new(torrent)?;
            let peers = torrent.discover_peers().await?;
            for peer in peers {
                println!("{:?}", peer);
            }
        }
        Commands::Handshake { torrent, peer_addr } => {
            let torrent = Torrent::new(torrent)?;
            let peer_id = torrent.peer_handshake(peer_addr).await?;
            println!("Peer ID: {}", peer_id);
        }
    }
    Ok(())
}
