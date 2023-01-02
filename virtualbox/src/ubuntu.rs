use std::io::{self, Write};
use std::process::Command;

use anyhow::Result;
use cloud::{write_seed_iso, Config, User};
use ssh_key::PrivateKey;

use crate::network;
use crate::ova;
use crate::ssh::private_key;
use crate::{manage, VIRTUALBOX_PATH};

// https://github.com/marysaka/mkisofs-rs
//https://wiki.debian.org/genisoimage
// https://gist.github.com/smoser/635897f845f7cb56c0a7ac3018a4f476#file-network-config-v1-yaml

// https://www.freedesktop.org/wiki/Software/systemd/PredictableNetworkInterfaceNames/

const UBUNTU_URL: &str =
    "https://cloud-images.ubuntu.com/jammy/current/jammy-server-cloudimg-amd64.ova";

pub async fn create(name: &str) -> Result<()> {
    let ova_path = ova::get("ubuntu-22_04", UBUNTU_URL).await?;

    println!("{}: import", name);
    let output = Command::new(manage::get_cmd())
        .arg("import")
        .arg(ova_path)
        .args(["--vsys", "0", "--vmname", name, "--basefolder"])
        .arg(VIRTUALBOX_PATH.to_path_buf())
        .output()?;
    io::stdout().write_all(&output.stdout)?;

    // config network

    let output = Command::new(manage::get_cmd())
        .args(["modifyvm", name, "--nic1", "nat"])
        .output()?;
    io::stdout().write_all(&output.stdout)?;

    let ssh_port = network::set_port_forward(name).await?;
    network::set_hostonly(name)?;

    // create seed.iso

    let host: u8 = (ssh_port - 2200).try_into()?;
    let key: PrivateKey = private_key().await?;
    let authorized_key = key.public_key().clone();

    let config = Config {
        hostname: String::from(name),
        host,
        user: User {
            ssh_key: Some(key),
            ssh_authorized_key: authorized_key,
        },
    };

    let seed = VIRTUALBOX_PATH.join(name).join("seed.iso");
    write_seed_iso(&seed, config)?;

    let output = Command::new(manage::get_cmd())
        .args([
            "storageattach",
            name,
            "--storagectl",
            "IDE",
            "--port",
            "0",
            "--device",
            "0",
            "--type",
            "dvddrive",
            "--medium",
        ])
        .arg(seed.to_path_buf())
        .output()?;
    io::stdout().write_all(&output.stdout)?;

    Ok(())
}
