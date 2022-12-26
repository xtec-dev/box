use anyhow::Result;
use std::io::{self, Write};
use std::process::Command;

use crate::ova;
use crate::ssh;
use crate::{manage, BOX_PATH};

// https://www.freedesktop.org/wiki/Software/systemd/PredictableNetworkInterfaceNames/
// https://docs.fedoraproject.org/en-US/fedora-coreos/provisioning-virtualbox/
// https://coreos.github.io/ignition/configuration-v3_2/

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

pub async fn create(name: &str) -> Result<()> {
    let ova_path = ova::get("coreos-37", COREOS_URL).await?;

    println!("{}: import", name);
    let output = Command::new(manage::get_cmd())
        .arg("import")
        .arg(ova_path)
        .args(["--vsys", "0", "--vmname", name, "--basefolder"])
        .arg(BOX_PATH.to_path_buf())
        .output()?;
    io::stdout().write_all(&output.stdout)?;

    let output = Command::new(manage::get_cmd())
        .args([
            "guestproperty",
            "set",
            name,
            "/Ignition/Config",
            IGNITION_CONFIG,
        ])
        .output()?;
    io::stdout().write_all(&output.stdout)?;

    ssh::set_port_forward(name)?;

    Ok(())
}
