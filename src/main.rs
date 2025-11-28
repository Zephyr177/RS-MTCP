mod config;
mod protocol;
mod connection_pool;
mod client;
mod server;

use clap::{Parser, Subcommand};
use log::info;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mtcp")]
#[command(about = "Multi-TCP connection aggregator with 0-RTT support", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run in server mode
    Server {
        /// Path to configuration file
        #[arg(short, long)]
        config: PathBuf,
    },
    /// Run in client mode
    Client {
        /// Path to configuration file
        #[arg(short, long)]
        config: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let cli = Cli::parse();

    match cli.command {
        Commands::Server { config } => {
            info!("Starting MTCP server");
            let cfg = config::Config::from_file(config)?;
            cfg.validate()?;
            
            if let Some(server_config) = cfg.server {
                let server = server::Server::new(server_config);
                server.run().await?;
            }
        }
        Commands::Client { config } => {
            info!("Starting MTCP client");
            let cfg = config::Config::from_file(config)?;
            cfg.validate()?;
            
            if let Some(client_config) = cfg.client {
                let client = client::Client::new(client_config).await?;
                client.run().await?;
            }
        }
    }

    Ok(())
}
