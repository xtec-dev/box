use std::path::Path;

use anyhow::Result;
use iso::{
    file_entry::FileEntry,
    utils::{self, LOGIC_SIZE_U32},
};

mod iso;

// https://cloudinit.readthedocs.io/en/latest/topics/examples.html
// https://gist.github.com/fardjad/a7e634d40f75dc29cff432e7372a1c93
// https://github.com/marysaka/mkisofs-rs

const USER_DATA: &str = r#"#cloud-config
groups:
  - ubuntu: [docker]
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

# NOTE: Cloud-init applies network settings on every boot by default. To retain network settings from first boot, add the following 'write_files' section:
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

    let metadata = format!(
        r#"local-hostname: {}
network-interfaces: |
  auto enp0s3
  iface enp0s3 inet dhcp"#,
        metadata.hostname
    );

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

    iso::create_iso(output, file_entries)?;
    Ok(())
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
