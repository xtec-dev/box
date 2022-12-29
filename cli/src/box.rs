use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use virtualbox::Machine;

#[derive(Parser)]
#[command(name = "box")]
#[command(author = "David de Mingo <david@optersoft.com>")]
#[command(version = "0.0.1")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a Virtual Machine.
    Create {
        /// The name of the Virtual Machine.
        name: String,

        /// Provider
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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Some(Commands::Create {
            name,
            provider,
            image,
        }) => create(name, provider, image).await,
        Some(Commands::Delete { name }) => delete(name).await,
        Some(Commands::List {}) => list(),
        Some(Commands::SSH { name }) => ssh(name).await,
        Some(Commands::Start { name }) => start(name).await,
        Some(Commands::Stop { name }) => stop(name).await,
        None => Ok(()),
    };

    if let Err(err) = result {
        println!("{}", err);
    }
}

async fn create(name: &String, provider: &Provider, image: &Image) -> Result<()> {
    let _p = provider;
    match image {
        Image::Coreos => virtualbox::create(name, virtualbox::Image::CoreOS).await,
        Image::Ubuntu => virtualbox::create(name, virtualbox::Image::Ubuntu).await,
    }
}

async fn delete(name: &String) -> Result<()> {
    let machine = Machine::new(name.clone());
    machine.delete().await
}

fn list() -> Result<()> {
    let mut vms = virtualbox::list_vms()?;
    vms.sort_by(|a, b| a.name.cmp(&b.name));

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["Name", "Status"]);

    for vm in vms {
        let info = vm.info()?;
        let state = match info.state() {
            Ok(state) => state.to_string(),
            Err(err) => format!("error: {}", err),
        };
        table.add_row(vec![vm.name, state]);
    }

    println!("{table}");

    Ok(())
}

async fn ssh(name: &String) -> Result<()> {
    let machine = Machine::new(name.clone());
    machine.ssh().await
}

async fn start(name: &String) -> Result<()> {
    let machine = Machine::new(name.clone());
    machine.start().await
}

async fn stop(name: &String) -> Result<()> {
    let machine = Machine::new(name.clone());
    machine.stop().await
}
