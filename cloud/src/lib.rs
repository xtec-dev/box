use std::{
    path::{Path, PathBuf},
    vec,
};

use anyhow::Result;

mod iso;

// https://cloudinit.readthedocs.io/en/latest/topics/examples.html
// https://gist.github.com/fardjad/a7e634d40f75dc29cff432e7372a1c93
// https://github.com/marysaka/mkisofs-rs

pub fn write_seed_iso(output: &Path) -> Result<()> {
    let path = PathBuf::from("/home/david/workspace/box/cloud/src/iso/seed");

    /*
       let mut file = OpenOptions::new()
           .create(true)
           .write(true)
           .truncate(true)
           .open(&path)
           .await?;
       file.write_all(SEED_ISO).await?;
    */

    let mut opt = iso::option::Opt {
        output: String::from(output.to_str().unwrap()),
        input_files: vec![path],
    };

    iso::create_iso(&mut opt)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use anyhow::Result;

    #[ignore]
    #[test]
    fn test_write_seed_iso() -> Result<()> {
        let output = Path::new("/home/david/workspace/box/virtualbox/init/seed.iso");
        write_seed_iso(&output)?;
        Ok(())
    }
}
