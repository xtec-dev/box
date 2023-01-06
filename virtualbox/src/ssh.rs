use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;
use once_cell::sync::Lazy;
use ssh_key::rand_core::OsRng;
use ssh_key::{Algorithm, LineEnding, PrivateKey};

use crate::network;

static KEY_PATH: Lazy<PathBuf> = Lazy::new(|| {
    home::home_dir()
        .expect("Home dir")
        .join(".ssh")
        .join("id_ed25519_box")
});

pub async fn connect(name: &str) -> Result<()> {
    if !KEY_PATH.exists() {
        private_key().await?;
    }

    let ip = network::get_hostonly(name)?;
    let host = format!("box@{}", ip);

    let mut child = Command::new("ssh")
        .args([
            "-i",
            &KEY_PATH.as_path().as_os_str().to_str().unwrap(),
            "-o",
            "UserKnownHostsFile=/dev/null",
            "-o",
            "StrictHostKeyChecking=no",
            &host,
        ])
        .spawn()
        .unwrap();
    let _asd = child.wait().unwrap();

    Ok(())
}

// key.public_key().to_openssh()?;
pub async fn private_key() -> Result<PrivateKey> {
    if KEY_PATH.exists() {
        let key = PrivateKey::read_openssh_file(&KEY_PATH)?;
        //.with_context(|| format!("file: {:?}", KEY_PATH.as_path()))?;
        return Ok(key);
    };

    let key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519)?;

    #[cfg(target_os = "windows")]
    let line_ending = LineEnding::CRLF;
    #[cfg(target_os = "linux")]
    let line_ending = LineEnding::LF;
    key.write_openssh_file(&KEY_PATH, line_ending)?;

    Ok(key)
}

#[cfg(test)]
mod tests {

    use ssh_key::PrivateKey;

    use super::*;

    #[test]
    fn test_key() -> Result<()> {
        let key: &str = r#"-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW
QyNTUxOQAAACCXTHXWq1zXA50wjtkxaBebqqw97InA6H5XbXncYZQ3VAAAAIi6f7S0un+0
tAAAAAtzc2gtZWQyNTUxOQAAACCXTHXWq1zXA50wjtkxaBebqqw97InA6H5XbXncYZQ3VA
AAAEBndCXRQsqznnNAG+XsDzdSF9SzhoUqBFp/lRpBJcVygJdMddarXNcDnTCO2TFoF5uq
rD3sicDofldtedxhlDdUAAAAA2JveAEC
-----END OPENSSH PRIVATE KEY-----
"#;

        let key = PrivateKey::from_openssh(key)?;
        assert_eq!(key.algorithm(), ssh_key::Algorithm::Ed25519);
        assert_eq!(key.comment(), "box");

        assert_eq!(
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJdMddarXNcDnTCO2TFoF5uqrD3sicDofldtedxhlDdU box",
            key.public_key().to_openssh().expect("ssh")
        );

        Ok(())
    }
}
