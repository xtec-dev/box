use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;
use once_cell::sync::Lazy;
use ssh_key::PrivateKey;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

static KEY_PATH: Lazy<PathBuf> = Lazy::new(|| {
    home::home_dir()
        .expect("Home dir")
        .join(".ssh")
        .join("id_ed25519_box")
});

pub async fn connect(id: u16) -> Result<()> {
    if !KEY_PATH.exists() {
        public_key().await?;
    }

    let port = format!("220{}", id);

    let mut child = Command::new("ssh")
        .args([
            "-p",
            &port,
            "-i",
            &KEY_PATH.as_path().as_os_str().to_str().unwrap(),
            "-o",
            "UserKnownHostsFile=/dev/null",
            "-o",
            "StrictHostKeyChecking=no",
            "box@127.0.0.1",
        ])
        .spawn()
        .unwrap();
    let _asd = child.wait().unwrap();

    Ok(())
}

pub async fn public_key() -> Result<String> {
    if !KEY_PATH.exists() {
        let key: &str = r#"-----BEGIN OPENSSH PRIVATE KEY-----
        b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW
        QyNTUxOQAAACCXTHXWq1zXA50wjtkxaBebqqw97InA6H5XbXncYZQ3VAAAAIi6f7S0un+0
        tAAAAAtzc2gtZWQyNTUxOQAAACCXTHXWq1zXA50wjtkxaBebqqw97InA6H5XbXncYZQ3VA
        AAAEBndCXRQsqznnNAG+XsDzdSF9SzhoUqBFp/lRpBJcVygJdMddarXNcDnTCO2TFoF5uq
        rD3sicDofldtedxhlDdUAAAAA2JveAEC
        -----END OPENSSH PRIVATE KEY-----
        "#;

        let mut file = OpenOptions::new()
            .create(false)
            .write(true)
            .open(KEY_PATH.as_path())
            .await?;
        file.write_all(key.as_bytes()).await?;
    }

    let mut file = File::open(KEY_PATH.as_path()).await?;
    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;
    let key = PrivateKey::from_openssh(contents)?;
    let public_key = key.public_key().to_openssh()?;

    Ok(public_key)
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

        assert_eq!(
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJdMddarXNcDnTCO2TFoF5uqrD3sicDofldtedxhlDdU box",
            key.public_key().to_openssh().expect("ssh")
        );

        Ok(())
    }
}

/*


Host box
    HostName 127.0.0.1
    IdentityFile ~/.ssh/id_ed25519_box
    User alumne
    Port 2201
    StrictHostKeyChecking no
    UserKnownHostsFile /dev/null
    */
