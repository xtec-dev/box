use std::path::Path;

use anyhow::{Context, Result};
use iso::{
    file_entry::FileEntry,
    utils::{self, LOGIC_SIZE_U32},
};
use ssh_key::{PrivateKey, PublicKey};

mod iso;

// https://cloudinit.readthedocs.io/en/latest/topics/datasources/nocloud.html

// https://cloudinit.readthedocs.io/en/latest/topics/examples.html
// https://github.com/marysaka/mkisofs-rs

// /var/log/cloud-init*

pub struct Config {
    pub hostname: String,
    pub host: u8,
    pub user: User,
}

pub struct User {
    pub ssh_key: Option<PrivateKey>,
    pub ssh_authorized_key: PublicKey,
}

pub fn write_seed_iso(output: &Path, config: Config) -> Result<()> {
    let output = String::from(output.to_str().unwrap());

    // https://cloudinit.readthedocs.io/en/latest/topics/network-config-format-v2.html#network-config-v2

    let mut file_entries = Vec::new();

    file_entries.push(meta_data(&config.hostname));
    file_entries.push(user_data(&config)?);
    file_entries.push(network_config(&config));

    iso::create_iso(output, file_entries)?;
    Ok(())
}

fn meta_data(hostname: &str) -> FileEntry {
    let metadata = format!(r#"local-hostname: {}"#, hostname);

    let entry = FileEntry {
        name: String::from("meta-data"),
        content: String::from(&metadata),
        size: metadata.len() as usize,
        lba: 0,
        aligned_size: utils::align_up(metadata.len() as i32, LOGIC_SIZE_U32 as i32) as usize,
    };
    entry
}

fn user_data(config: &Config) -> Result<FileEntry> {
    let ssh_authorized_key = config
        .user
        .ssh_authorized_key
        .to_openssh()
        .context("cloud: user-data: authorized key")?;

    // key.to_openssh(ssh_key::LineEnding::LF)?;

    let data = format!(
        r#"#cloud-config
users:
  - name: box
    groups: sudo, docker
    sudo: ["ALL=(ALL) NOPASSWD:ALL"]
    plain_text_passwd: password
    lock_passwd: false
    shell: /bin/bash
    #ssh_pwauth: true
    ssh_authorized_keys:
      - {}
"#,
        ssh_authorized_key
    );

    let entry = FileEntry {
        name: String::from("user-data"),
        content: String::from(&data),
        size: data.len() as usize,
        lba: 0,
        aligned_size: utils::align_up(data.len() as i32, LOGIC_SIZE_U32 as i32) as usize,
    };

    Ok(entry)
}

/*
  Network configuration can be provided to cloud-init in Networking Config Version 2 by providing that YAML formatted data in a file named network-config. If found, this file will override a network-interfaces file.
*/
fn network_config(config: &Config) -> FileEntry {
    let config = format!(
        r#"version: 2
ethernets:
  enp0s3:
    dhcp4: true
  enp0s8:
    addresses:
      - 192.168.56.{}/24
"#,
        config.host
    );

    let entry = FileEntry {
        name: String::from("network-config"),
        content: String::from(&config),
        size: config.len() as usize,
        lba: 0,
        aligned_size: utils::align_up(config.len() as i32, LOGIC_SIZE_U32 as i32) as usize,
    };

    entry
}

#[cfg(test)]
mod tests {

    use anyhow::Result;
    use rand_core::OsRng;
    use ssh_key::{Algorithm, PrivateKey};
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_write_seed_iso() -> Result<()> {
        let key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519)?;
        let authorized_key = key.public_key().clone();

        let config = Config {
            hostname: String::from("test"),
            host: 1,
            user: User {
                ssh_key: Some(key),
                ssh_authorized_key: authorized_key,
            },
        };

        let file = NamedTempFile::new()?;
        write_seed_iso(&file.path(), config)?;
        Ok(())
    }
}
