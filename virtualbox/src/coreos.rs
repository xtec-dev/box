use anyhow::Result;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

use crate::ova;

// https://www.freedesktop.org/wiki/Software/systemd/PredictableNetworkInterfaceNames/
// https://docs.fedoraproject.org/en-US/fedora-coreos/provisioning-virtualbox/

// https://builds.coreos.fedoraproject.org/prod/streams/stable/builds/37.20221127.3.0/x86_64/fedora-coreos-37.20221127.3.0-virtualbox.x86_64.ova

//VBoxManage import --vsys 0 --vmname "$VM_NAME" fedora-coreos-37.20221127.3.0-virtualbox.x86_64.ova

// IGN_PATH="/path/to/config.ign"
// VBoxManage guestproperty set "$VM_NAME" /Ignition/Config "$(cat $IGN_PATH)"

// https://coreos.github.io/ignition/configuration-v3_2/

use crate::{manage, Machine, BOX_PATH};

const COREOS_URL: &str = "https://builds.coreos.fedoraproject.org/prod/streams/stable/builds/37.20221127.3.0/x86_64/fedora-coreos-37.20221127.3.0-virtualbox.x86_64.ova";

const IGNITION_CONFIG: &str = r#"{
    "ignition": { "version": "3.0.0" },
    "passwd": {
      "users": [
        {
          "name": "box",
          "passwordHash": "$y$j9T$BAlET20ZhfuQ.YzttOAaA.$8O8Fb/0UMSq5TPyufNVGffUrUYiazipQglTTo4VN.iB",
          "sshAuthorizedKeys": [
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJdMddarXNcDnTCO2TFoF5uqrD3sicDofldtedxhlDdU box"
          ]
        }
      ]
    }
  }"#;

pub async fn import(machine: &Machine) -> Result<()> {
    let ova_path = ova::get("coreos-37", COREOS_URL).await?;

    println!("{}: import", machine);
    let output = Command::new(manage::get_cmd())
        .arg("import")
        .arg(ova_path)
        .args(["--vsys", "0", "--vmname", machine.as_ref(), "--basefolder"])
        .arg(BOX_PATH.to_path_buf())
        .output()?;
    io::stdout().write_all(&output.stdout)?;

    // VBoxManage guestproperty set box-5 /Ignition/Config "$(cat config.ign)"

    let output = Command::new(manage::get_cmd())
        .args([
            "guestproperty",
            "set",
            &machine.name,
            "/Ignition/Config",
            IGNITION_CONFIG,
        ])
        .output()?;
    io::stdout().write_all(&output.stdout)?;

    // TODO shared with ubuntu
    let rule = format!("ssh,tcp,127.0.0.1,220{},,22", machine.id());

    let output = Command::new(manage::get_cmd())
        .args(["modifyvm", machine.as_ref(), "--natpf1", &rule])
        .output()?;
    io::stdout().write_all(&output.stdout)?;

    Ok(())
}
