use anyhow::Result;
use std::io::{self, Write};
use std::process::Command;

use crate::ova;
use crate::ssh;
use crate::{manage, BOX_PATH};



/*

To generate a secure password hash, use mkpasswd from the whois package. Your Linux distro may ship a different mkpasswd implementation; you can ensure you’re using the correct one by running it from a container:

$ podman run -ti --rm quay.io/coreos/mkpasswd --method=yescrypt
Password:
$y$j9T$A0Y3wwVOKP69S.1K/zYGN.$S596l11UGH3XjN...
The yescrypt hashing method is recommended for new passwords. For more details on hashing methods, see man 5 crypt.

The configured password will be accepted for local authentication at the console. By default, Fedora CoreOS does not allow password authentication via SSH.

https://docs.fedoraproject.org/en-US/fedora-coreos/authentication/#_enabling_ssh_password_authentication

*/

// https://docs.fedoraproject.org/en-US/fedora-coreos/authentication/

// https://www.freedesktop.org/wiki/Software/systemd/PredictableNetworkInterfaceNames/

// https://docs.fedoraproject.org/en-US/fedora-coreos/provisioning-virtualbox/
// https://coreos.github.io/ignition/configuration-v3_2/

const COREOS_URL: &str = "https://builds.coreos.fedoraproject.org/prod/streams/stable/builds/37.20221211.3.0/x86_64/fedora-coreos-37.20221211.3.0-virtualbox.x86_64.ova";

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

    let ignition = new_ignition(name); 

    let output = Command::new(manage::get_cmd())
        .args([
            "guestproperty",
            "set",
            name,
            "/Ignition/Config",
            &ignition,
        ])
        .output()?;
    io::stdout().write_all(&output.stdout)?;

    ssh::set_port_forward(name).await?;

    Ok(())
}

fn new_ignition(hostname: &str) -> String {

  // podman run -i --rm quay.io/coreos/butane:release --pretty --strict < config.bu > config.ign


  // TODO fix zincati
  //sudo systemctl disable --now zincati.service
  // https://github.com/coreos/fedora-coreos-tracker/issues/392

  let ignition = format!(r#"{{
  "ignition": {{ "version": "3.0.0" }},
  "passwd": {{
    "users": [
      {{
        "name": "box",
        "groups": [ "docker", "wheel"],
        "passwordHash": "$y$j9T$BAlET20ZhfuQ.YzttOAaA.$8O8Fb/0UMSq5TPyufNVGffUrUYiazipQglTTo4VN.iB",
        "sshAuthorizedKeys": [
          "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJdMddarXNcDnTCO2TFoF5uqrD3sicDofldtedxhlDdU box"
        ]
      }}
    ]
  }},
  "storage": {{
    "files": [
      {{
        "path": "/etc/hostname",
        "contents": {{
          "compression": "",
          "source": "data:,{}%0A"
        }},
        "mode": 420
      }}
    ]
  }}
}}"#, hostname);

  ignition
}
