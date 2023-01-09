use anyhow::Result;

use std::io::{self, Write};
use std::process::Command;

use crate::ssh::ssh_authorized_key;
use crate::{manage, VIRTUALBOX_PATH};
use crate::{network, ova};

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
    let ova_path = ova::get("coreos-37-20221211", COREOS_URL).await?;

    println!("{}: import", name);
    let output = Command::new(manage::get_cmd())
        .arg("import")
        .arg(ova_path)
        .args(["--vsys", "0", "--vmname", name, "--basefolder"])
        .arg(VIRTUALBOX_PATH.as_path())
        .output()?;
    io::stdout().write_all(&output.stdout)?;

    network::set_hostonly(name).await?;
    let host = network::get_host(name)?.expect("host no found");

    let ignition = new_ignition(name, host).await?;

    let output = Command::new(manage::get_cmd())
        .args(["guestproperty", "set", name, "/Ignition/Config", &ignition])
        .output()?;
    io::stdout().write_all(&output.stdout)?;

    //network::set_port_forward(name).await?;

    Ok(())
}

async fn new_ignition(hostname: &str, host: u8) -> Result<String> {
    // podman run -i --rm quay.io/coreos/butane:release --pretty --strict < config.bu > config.ign

    // TODO fix zincati
    //sudo systemctl disable --now zincati.service
    // https://github.com/coreos/fedora-coreos-tracker/issues/392

    let ssh_authorized_key = ssh_authorized_key().await?;

    let ignition = format!(
        r#"{{
  "ignition": {{ "version": "3.0.0" }},
  "passwd": {{
    "users": [
      {{
        "name": "box",
        "groups": [ "docker", "wheel"],
        "passwordHash": "$y$j9T$BAlET20ZhfuQ.YzttOAaA.$8O8Fb/0UMSq5TPyufNVGffUrUYiazipQglTTo4VN.iB",
        "sshAuthorizedKeys": [
          "{}"
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
      }},
      {{
        "path": "/etc/NetworkManager/system-connections/enp0s8.nmconnection",
        "contents": {{
          "compression": "",
          "source": "data:,%5Bconnection%5D%0Aid%3Denp0s8%0Atype%3Dethernet%0Ainterface-name%3Denp0s8%0A%5Bipv4%5D%0Aaddress1%3D192.168.56.{}%0Amay-fail%3Dfalse%0Amethod%3Dmanual%0A"
        }},
        "mode": 384
      }}
    ]
  }}
}}"#,
        ssh_authorized_key, hostname, host
    );

    Ok(ignition)
}
