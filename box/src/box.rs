mod google;
mod hetzner;

use std::ops::RangeInclusive;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tokio::runtime::Runtime;
use virtualbox::Machine;

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
    /// Lists all virtual machines currently registered with VirtualBox.
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

fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::List {}) => list(),
        Some(Commands::Start { id }) => start(*id),
        Some(Commands::Stop { id }) => {
            let rt = Runtime::new().expect("tokio runtime can be initialized");
            rt.block_on(async move {
                match virtualbox::stop(*id).await {
                    Ok(v) => v,
                    Err(e) => return println!("could not stop vm {}, reason: {}", id, e),
                };
            });
            Ok(())
        }
        None => Ok(()),
    }
}

fn list() -> Result<()> {
    let vms = virtualbox::list_vms()?;
    for vm in vms {
        let info = vm.info()?;
        match info {
            None => println!("{}", vm.name),
            Some(info) => println!("{} {:?}", vm.name, info.get_state()),
        }
    }
    Ok(())

    /*

     // Get list of all known virtual machines in system
        let lst = vboxhelper::get_vm_list().expect("Unable to get VM list");

        // Get a HashSet containing all known _running_ virtual machines
        let running = {
            let mut set = HashSet::new();
            for (_, uuid) in vboxhelper::get_running_vms_list().expect("Unable to get VM list") {
                set.insert(uuid);
            }

            set
        };

        // Find the longest virtual machine name, to make make output visually
        // stunning.
        let mut max_len = 0;
        for (nm, _) in &lst {
            if nm.len() > max_len {
                max_len = nm.len();
            }
        }

        // Display a list of all virtual machines, and marking the running ones.
        for (nm, uuid) in &lst {
            let runstate = if running.contains(&uuid) {
                " [running]"
            } else {
                ""
            };

            println!("{:width$}  {}{}", nm, uuid, runstate, width = max_len);
        }
    */
}

fn start(id: u16) -> Result<()> {
    let name = format!("xtec-{}", id);
    let machine = Machine::new(name);
    let rt = Runtime::new()?;
    rt.block_on(async move {
        if let Err(err) = machine.start().await {
            println!("{}: {}", machine.name, err);
        }
    });
    Ok(())
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
