use clap::{Parser, Subcommand};
use dirs::home_dir;
use solana_sdk::signature::{read_keypair_file, Keypair};
use std::fs;
use std::path::PathBuf;

mod generate_key;
mod mine;
mod protomine;

const CONFIG_FILE: &str = "keypair_list";

/// A command line interface tool for pooling power to submit hashes for proportional ORE rewards
#[derive(Parser, Debug)]
#[command(version, author, about, long_about = None)]
struct Args {
    #[arg(
        long,
        value_name = "SERVER_URL",
        help = "Host name and port of your private pool server to connect to, it can also be your LAN ip address:port like: 172.xxx.xx.xxx:3000, 192.xxx.xx.xxx:3000",
        default_value = "orepool.miraland.io:3000"
    )]
    url: String,

    #[arg(
        long,
        value_name = "KEYPAIR_PATH",
        help = "Filepath to keypair to use",
        default_value = "~/.config/solana/id.json"
    )]
    keypair: String,

    #[arg(long, short, action, help = "Use unsecure http connection instead of https.")]
    use_http: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Connect to pool and start mining. (Default Implementation)")]
    Mine(mine::MineArgs),
    #[command(about = "Connect to pool and start mining. (Protomine Implementation)")]
    Protomine(protomine::MineArgs),
    #[command(about = "Generate a new solana keypair for mining.")]
    Keygen,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let base_url = args.url;
    let unsecure_conn = args.use_http;

    // Does the config file exist? If not, create one
    let config_path = PathBuf::from(CONFIG_FILE);
    if !config_path.exists() {
        fs::File::create(&config_path).expect("Failed to create configuration file.");
    }

    // let key = read_keypair_file(args.keypair.clone())
    //     .expect(&format!("Failed to load keypair from file: {}", args.keypair));
    let key: Keypair;
    // Check if keypair path is provided or fallback to the default
    let keypair_path = expand_tilde(&args.keypair);
    let keypair_exists = PathBuf::from(&keypair_path).exists();

    if keypair_exists {
        // Keypair path is provided and exists, proceed directly
        key = read_keypair_file(&keypair_path)
            .expect(&format!("Failed to load keypair from file: {}", keypair_path));
    } else {
        // The keypair does not exist, exit program
        eprintln!(
            "Keypair not found. Please provide one or generate a new keypair with keygen sub-command. Exiting program."
        );
        std::process::exit(0);
    }

    match args.command {
        Commands::Mine(args) => {
            mine::mine(args, key, base_url, unsecure_conn).await;
        },
        Commands::Protomine(args) => {
            protomine::protomine(args, key, base_url, unsecure_conn).await;
        },
        Commands::Keygen => {
            generate_key::generate_key();
        },
    }
}

fn expand_tilde(path: &str) -> String {
    if path.starts_with("~") {
        if let Some(home_dir) = home_dir() {
            return path.replacen("~", &home_dir.to_string_lossy(), 1);
        }
    }
    path.to_string()
}
