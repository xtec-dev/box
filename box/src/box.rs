use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
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
    /// Create a Virtual Machine.
    Create {
        name: String,

        ///
        #[arg(value_enum,short,long, default_value_t = Provider::VirtualBox)]
        provider: Provider,

        /// OS
        #[arg(value_enum, short,long,default_value_t = Image::Ubuntu)]
        image: Image,
    },

    /// Delete a VM.
    Delete {
        /// The name of the Virtual Machine.
        #[arg()]
        name: String,
    },

    /// Lists all virtual machines currently registered with VirtualBox.
    List {},

    /// Start a virtual machine
    SSH {
        /// The name of the Virtual Machine.
        #[arg()]
        name: String,
    },

    /// Start a stopped VM.
    Start {
        /// The name of the Virtual Machine.
        #[arg()]
        name: String,
    },

    /// Power off (stop) a running VM.
    Stop {
        /// The name of the Virtual Machine.
        #[arg()]
        name: String,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Image {
    /// Ubuntu 22.04
    Ubuntu,
    /// CoreOS 37
    Coreos,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Provider {
    /// VirtualBox
    VirtualBox,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Create {
            name,
            provider,
            image,
        }) => {
            println!("{} {:?} {:?}", name, provider, image);
            Ok(())
        }
        Some(Commands::Delete { name }) => delete(name),
        Some(Commands::List {}) => list(),
        Some(Commands::SSH { name }) => ssh(name),
        Some(Commands::Start { name }) => start(name),
        Some(Commands::Stop { name }) => stop(name),
        None => Ok(()),
    }
}

fn delete(name: &String) -> Result<()> {
    let machine = Machine::new(name.clone());
    let rt = Runtime::new()?;
    rt.block_on(async move {
        if let Err(err) = machine.delete().await {
            println!("{}: {}", machine.name, err);
        }
    });
    Ok(())
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
}

fn ssh(name: &String) -> Result<()> {
    let machine = Machine::new(name.clone());
    let rt = Runtime::new()?;
    rt.block_on(async move {
        if let Err(err) = machine.ssh().await {
            println!("{}: {}", machine.name, err);
        }
    });
    Ok(())
}

fn start(name: &String) -> Result<()> {
    let machine = Machine::new(name.clone());
    let rt = Runtime::new()?;
    rt.block_on(async move {
        if let Err(err) = machine.start().await {
            println!("{}: {}", machine.name, err);
        }
    });
    Ok(())
}

fn stop(name: &String) -> Result<()> {
    let machine = Machine::new(name.clone());
    let rt = Runtime::new()?;
    rt.block_on(async move {
        if let Err(err) = machine.stop().await {
            println!("{}: {}", machine.name, err);
        }
    });
    Ok(())
}
