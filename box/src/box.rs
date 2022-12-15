mod google;
mod hetzner;

use std::ops::RangeInclusive;

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
    /// Lists all xtec virtual machines currently registered with VirtualBox.
    List {},

    /// Start a virtual machine
    Start {
        /// Virtual machine id, from 1 to 9
        #[arg(value_parser = id_in_range)]
        id: u16,
    },

    /// Stop a virtual machine
    Stop {
        /// Virtual machine id, from 1 to 9
        #[arg(value_parser = id_in_range)]
        id: u16,
    },
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::List {}) => {
            let rt = Runtime::new().expect("tokio runtime can be initialized");
            rt.block_on(async move {
                match virtualbox::list() {
                    Ok(()) => (),
                    Err(e) => return println!("could not list vms, reason: {}", e),
                };
            });
        }
        Some(Commands::Start { id }) => {
            let rt = Runtime::new().expect("tokio runtime can be initialized");
            rt.block_on(async move {
                match virtualbox::start(*id).await {
                    Ok(v) => v,
                    Err(e) => return println!("could not start vm {}, reason: {}", id, e),
                };
            });
        }
        Some(Commands::Stop { id }) => {
            let rt = Runtime::new().expect("tokio runtime can be initialized");
            rt.block_on(async move {
                match virtualbox::stop(*id).await {
                    Ok(v) => v,
                    Err(e) => return println!("could not stop vm {}, reason: {}", id, e),
                };
            });
        }
        None => {}
    }
}

const ID_RANGE: RangeInclusive<usize> = 1..=9;

fn id_in_range(s: &str) -> Result<u16, String> {
    let port: usize = s
        .parse()
        .map_err(|_| format!("`{}` isn't a id number", s))?;
    if ID_RANGE.contains(&port) {
        Ok(port as u16)
    } else {
        Err(format!(
            "Id not in range {}-{}",
            ID_RANGE.start(),
            ID_RANGE.end()
        ))
    }
}

/*
#[tokio::main]
async fn main() {
    virtualbox::start(1).await.expect("machine");
}*/
