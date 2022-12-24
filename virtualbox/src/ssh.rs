use std::process::Command;

use anyhow::Result;

pub fn connect(id: u16) -> Result<()> {

    let i = home::home_dir().expect("Home dir").join(".ssh").join("id_ed25519_box");

    let port = format!("220{}", id);

    let mut child = Command::new("ssh")
        .args([
            "-p",
            &port,
            "-i",
            &i.into_os_string().into_string().unwrap(),
            "-o",
            "UserKnownHostsFile=/dev/null",
            "-o",
            "StrictHostKeyChecking=no",
            "alumne@127.0.0.1",
        ])
        .spawn()
        .unwrap();
    let _asd = child.wait().unwrap();

    Ok(())
}