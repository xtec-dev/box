mod google;
mod hetzner;

use clap::{Parser, Subcommand};
use tokio::runtime::Runtime;

#[derive(Parser)]
#[command(name = "box")]
#[command(author = "David de Mingo <david@optersoft.com>")]
#[command(version = "0.1")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Start { id: Option<u8> },
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Start { id }) => {
            let id = id.unwrap_or(1);
            let rt = Runtime::new().expect("tokio runtime can be initialized");
            rt.block_on(async move {
                match virtualbox::start(id).await {
                    Ok(v) => v,
                    Err(e) => return println!("could not start vm, reason: {}", e),
                };
            });

            //
            println!("start {:?}", id)
        }
        None => {}
    }
}

/*
#[tokio::main]
async fn main() {
    virtualbox::start(1).await.expect("machine");
}*/
