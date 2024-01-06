mod decoder;

use std::{env, str::FromStr};

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

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let command: Result<Command, ()> = args[1].parse();

        match command {
            Ok(cmd) => {
                println!("Command: {:?}", cmd);
                let bencoded_value = decoder::BencodedValue::decode(&args[2]);
                println!("Decoded value: {}", bencoded_value.value.to_string());
            }
            Err(_) => println!("Invalid command"),
        }
    }
}
