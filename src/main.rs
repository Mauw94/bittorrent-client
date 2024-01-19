mod decoder;
mod peer;
mod torrent;
mod tracker;
use clap::{Parser, Subcommand};
use std::{fs, net::SocketAddrV4, path::PathBuf};
use torrent::Torrent;

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
#[clap(rename_all = "snake_case")]
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
    DownloadPiece {
        output_path: PathBuf,
        torrent: PathBuf,
        piece: u32,
    },
    TestOutput {
        output_path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Decode { encoded_bencode } => {
            let bencoded_value = decoder::BencodedValue::decode(&encoded_bencode);
            println!("Decoded value: {}", bencoded_value.value.to_string());
            return Ok(());
        }
        Commands::Info { torrent } => {
            let torrent = Torrent::new(torrent)?;
            println!("Name, {}", torrent.info.name);
            println!("Tracker URL: {}", torrent.announce);
            println!("Length: {}", torrent.info.length);
            println!("Info Hash: {}", torrent.info_hash_hex()?);
            println!("Piece Length: {}", torrent.info.piece_length);
        }
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
        Commands::DownloadPiece {
            output_path,
            torrent,
            piece,
        } => {
            let torrent = Torrent::new(torrent)?;
            let data = torrent.download_piece(piece).await?;
            fs::write(output_path, data).unwrap();
        }
        Commands::TestOutput { output_path } => {
            fs::write(output_path, String::from("this is a test")).unwrap();
        }
    }
    Ok(())
}
