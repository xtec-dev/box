use anyhow::Result;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

use crate::ova;
use crate::{manage, Machine, BOX_PATH};

// https://github.com/marysaka/mkisofs-rs
//https://wiki.debian.org/genisoimage

// https://www.freedesktop.org/wiki/Software/systemd/PredictableNetworkInterfaceNames/

const UBUNTU_URL: &str =
    "https://cloud-images.ubuntu.com/jammy/current/jammy-server-cloudimg-amd64.ova";

const INIT_ISO: &[u8] = include_bytes!("../init/init.iso");

pub async fn import(machine: &Machine) -> Result<()> {
    let ova_path = ova::get("ubuntu-22_04", UBUNTU_URL).await?;

    println!("{}: import", machine);
    let output = Command::new(manage::get_cmd())
        .arg("import")
        .arg(ova_path)
        .args(["--vsys", "0", "--vmname", machine.as_ref(), "--basefolder"])
        .arg(BOX_PATH.to_path_buf())
        .output()?;
    io::stdout().write_all(&output.stdout)?;

    let init = BOX_PATH.join("init.iso");
    make_init(&init).await?;

    let output = Command::new(manage::get_cmd())
        .args([
            "storageattach",
            machine.as_ref(),
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
        .arg(init.to_path_buf())
        .output()?;
    io::stdout().write_all(&output.stdout)?;

    // VBoxManage.exe storageattach "<uuid|vmname>" --storagectl IDE --port 0 --device 0 --medium "none"

    let output = Command::new(manage::get_cmd())
        .args(["modifyvm", machine.as_ref(), "--nic1", "nat"])
        .output()?;
    io::stdout().write_all(&output.stdout)?;

    let rule = format!("ssh,tcp,127.0.0.1,220{},,22", machine.id());

    let output = Command::new(manage::get_cmd())
        .args(["modifyvm", machine.as_ref(), "--natpf1", &rule])
        .output()?;
    io::stdout().write_all(&output.stdout)?;

    Ok(())
}

async fn make_init(path: &Path) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .await?;
    file.write_all(INIT_ISO).await?;

    Ok(())
}
