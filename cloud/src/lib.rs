use std::path::Path;

use anyhow::Result;
use iso::{
    file_entry::FileEntry,
    utils::{self, LOGIC_SIZE_U32},
};

mod iso;

// https://cloudinit.readthedocs.io/en/latest/topics/datasources/nocloud.html

// https://cloudinit.readthedocs.io/en/latest/topics/examples.html
// https://github.com/marysaka/mkisofs-rs

// /var/log/cloud-init*

const USER_DATA: &str = r#"#cloud-config
users:
  - name: box
    groups: sudo, docker
    sudo: ["ALL=(ALL) NOPASSWD:ALL"]
    plain_text_passwd: password
    lock_passwd: false
    shell: /bin/bash
    #ssh_pwauth: true
    ssh_authorized_keys:
      - ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJdMddarXNcDnTCO2TFoF5uqrD3sicDofldtedxhlDdU box

write_files:
  - path: /etc/cloud/cloud.cfg.d/80_disable_network_after_firstboot.cfg
    content: |
      # Disable network configuration after first boot
      network:
        config: disabled
"#;

pub struct MetaData {
    pub hostname: String,
}

pub fn write_seed_iso(output: &Path, metadata: MetaData) -> Result<()> {
    let output = String::from(output.to_str().unwrap());

    // https://cloudinit.readthedocs.io/en/latest/topics/network-config-format-v2.html#network-config-v2
    let metadata = format!(r#"local-hostname: {}"#, metadata.hostname);

    let mut file_entries = Vec::new();
    let entry = FileEntry {
        name: String::from("meta-data"),
        content: String::from(&metadata),
        size: metadata.len() as usize,
        lba: 0,
        aligned_size: utils::align_up(metadata.len() as i32, LOGIC_SIZE_U32 as i32) as usize,
    };
    file_entries.push(entry);

    let entry = FileEntry {
        name: String::from("user-data"),
        content: String::from(USER_DATA),
        size: USER_DATA.len() as usize,
        lba: 0,
        aligned_size: utils::align_up(USER_DATA.len() as i32, LOGIC_SIZE_U32 as i32) as usize,
    };
    file_entries.push(entry);

    file_entries.push(network_config());

    iso::create_iso(output, file_entries)?;
    Ok(())
}

/*
  Network configuration can be provided to cloud-init in Networking Config Version 2 by providing that YAML formatted data in a file named network-config. If found, this file will override a network-interfaces file.
*/
fn network_config() -> FileEntry {
    let config = format!(
        r#"version: 2
ethernets:
  enp0s3:
    dhcp4: true
  enp0s8:
    addresses:
      - 192.168.56.{}/24
"#,
        101
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

    #[ignore]
    #[test]
    fn test_write_seed_iso() -> Result<()> {
        //let output = Path::new("/home/david/workspace/box/virtualbox/init/seed.iso");
        //write_seed_iso(&output)?;
        Ok(())
    }
}
