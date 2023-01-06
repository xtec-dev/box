use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use virtualbox::Machine;

/*

.name(env!("CARGO_PKG_NAME"))
    .about(env!("CARGO_PKG_DESCRIPTION"))
    .version(env!("CARGO_PKG_VERSION"))
    arg_required_else_help(true),
*/

#[derive(Parser)]
#[command(name = "box")]
#[command(author = "David de Mingo <david@optersoft.com>")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a stopped virtual machine and config ssh ...
    Code { name: String },
    /// Create a Virtual Machine.
    Create {
        /// The name of the Virtual Machine.
        name: String,

        /// Provider
        #[arg(value_enum,short,long, default_value_t = Provider::Virtualbox)]
        provider: Provider,
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
enum Provider {
    /// VirtualBox
    Virtualbox,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Some(Commands::Code { name }) => code(name).await,
        Some(Commands::Create { name, provider }) => create(name, provider).await,
        Some(Commands::Delete { name }) => delete(name).await,
        Some(Commands::List {}) => list(),
        Some(Commands::SSH { name }) => ssh(name).await,
        Some(Commands::Start { name }) => start(name).await,
        Some(Commands::Stop { name }) => stop(name).await,
        None => {
            if let Err(err) = Cli::command().print_help() {
                println!("error: {}", err);
            };
            Ok(())
        }
    };

    if let Err(err) = result {
        println!("error: {}", err);
    }
}

async fn code(name: &String) -> Result<()> {
    code::start(name).await
}

async fn create(name: &String, provider: &Provider) -> Result<()> {
    let _p = provider;
    virtualbox::create(name).await
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
    table.set_header(vec!["Name", "Status", "IP"]);

    for vm in vms {
        let info = vm.info()?;
        let state = match info.state() {
            Ok(state) => state.to_string(),
            Err(err) => format!("error: {}", err),
        };
        let ip = match vm.get_ip() {
            Ok(ip) => ip,
            Err(err) => format!("error: {}", err),
        };
        table.add_row(vec![vm.name, state, ip]);
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
