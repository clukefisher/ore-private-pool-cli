use clap::{Parser, Subcommand};
use solana_sdk::signature::read_keypair_file;

mod mine;
mod protomine;

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

    #[arg(
        long,
        short,
        action,
        help = "Use unsecure http connection instead of https."
    )]
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
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let base_url = args.url;
    let unsecure_conn = args.use_http;
    let key = read_keypair_file(args.keypair.clone()).expect(&format!(
        "Failed to load keypair from file: {}",
        args.keypair
    ));
    match args.command {
        Commands::Mine(args) => {
            mine::mine(args, key, base_url, unsecure_conn).await;
        }
        Commands::Protomine(args) => {
            protomine::mine(args, key, base_url, unsecure_conn).await;
        }
    }
}
