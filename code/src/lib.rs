use anyhow::Result;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use virtualbox::Machine;

pub async fn start(name: &String) -> Result<()> {
    let machine = Machine::new(name.clone());
    let port = machine.info()?.ssh_port()?;

    let ssh_path = home::home_dir().expect("user home").join(".ssh");
    let identity_path = ssh_path.join("id_ed25519_box");

    let config = format!(
        r#"Host code
    HostName 127.0.0.1
    IdentityFile {}
    User box
    Port {}
    StrictHostKeyChecking no
    UserKnownHostsFile /dev/null"#,
        identity_path.as_os_str().to_str().unwrap(),
        port
    );

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(ssh_path.join("config"))
        .await?;
    file.write_all(config.as_bytes()).await?;

    machine.start().await?;

    Ok(())
}
