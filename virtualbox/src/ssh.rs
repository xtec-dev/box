use std::process::Command;

use anyhow::Result;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

const PK: &str = r#"-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW
QyNTUxOQAAACCXTHXWq1zXA50wjtkxaBebqqw97InA6H5XbXncYZQ3VAAAAIi6f7S0un+0
tAAAAAtzc2gtZWQyNTUxOQAAACCXTHXWq1zXA50wjtkxaBebqqw97InA6H5XbXncYZQ3VA
AAAEBndCXRQsqznnNAG+XsDzdSF9SzhoUqBFp/lRpBJcVygJdMddarXNcDnTCO2TFoF5uq
rD3sicDofldtedxhlDdUAAAAA2JveAEC
-----END OPENSSH PRIVATE KEY-----"#;

pub async fn connect(id: u16) -> Result<()> {
    let pk = home::home_dir()
        .expect("Home dir")
        .join(".ssh")
        .join("id_ed25519_box");

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&pk)
        .await?;
    file.write_all(PK.as_bytes()).await?;

    let port = format!("220{}", id);

    let mut child = Command::new("ssh")
        .args([
            "-p",
            &port,
            "-i",
            &pk.into_os_string().into_string().unwrap(),
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

/*


Host box
    HostName 127.0.0.1
    IdentityFile ~/.ssh/id_ed25519_box
    User alumne
    Port 2201
    StrictHostKeyChecking no
    UserKnownHostsFile /dev/null
    */
