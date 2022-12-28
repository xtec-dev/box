use std::{path::PathBuf, vec};

use anyhow::Result;

mod iso;

// https://cloudinit.readthedocs.io/en/latest/topics/examples.html
// https://gist.github.com/fardjad/a7e634d40f75dc29cff432e7372a1c93
// https://github.com/marysaka/mkisofs-rs

pub fn cloud() -> Result<()> {
    let path = PathBuf::from("/home/david/workspace/box/cloud/src/iso/seed");

    let mut opt = iso::option::Opt {
        output: String::from("/home/david/workspace/box/virtualbox/init/seed.iso"),
        eltorito_opt: iso::option::ElToritoOpt {
            eltorito_boot: None,
            no_emu_boot: false,
            no_boot: false,
            boot_info_table: false,
            grub2_boot_info: false,
        },
        embedded_boot: None,
        grub2_mbr: None,
        boot_load_size: 4,
        protective_msdos_label: false,
        input_files: vec![path],
    };

    iso::create_iso(&mut opt)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_create() -> Result<()> {
        cloud()?;
        Ok(())
    }
}
