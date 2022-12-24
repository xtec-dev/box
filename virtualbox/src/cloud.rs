use anyhow::{Context, Result};
use futures_util::StreamExt;
use reqwest::Client;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::{cmp::min, path::Path};

use indicatif::{ProgressBar, ProgressStyle};
use tokio::fs::{OpenOptions};
use tokio::io::AsyncWriteExt;

// https://github.com/marysaka/mkisofs-rs
//https://wiki.debian.org/genisoimage

// https://www.freedesktop.org/wiki/Software/systemd/PredictableNetworkInterfaceNames/

use crate::{manage, BOX_PATH};

const UBUNTU: &str =
    "https://cloud-images.ubuntu.com/jammy/current/jammy-server-cloudimg-amd64.ova";

const INIT_ISO: &[u8] = include_bytes!("../init/init.iso");



pub async fn import(name: &str) -> Result<()> {
    let ova_path = get_ova().await?;

    println!("{}: import", name);
    let output = Command::new(manage::get_cmd())
        .arg("import")
        .arg(ova_path)
        .args(["--vsys", "0", "--vmname", name, "--basefolder"])
        .arg(BOX_PATH.to_path_buf())
        .output()?;
    io::stdout().write_all(&output.stdout)?;

    let init = BOX_PATH.join("init.iso");
    make_init(&init).await?;

    let output = Command::new(manage::get_cmd())
        .args([
            "storageattach",
            name,
            "--storagectl",
            "IDE",
            "--port",
            "0",
            "--device",
            "0",
            "--type",
            "dvddrive",
            "--medium",
        ])
        .arg(init.to_path_buf())
        .output()?;
    io::stdout().write_all(&output.stdout)?;


    // VBoxManage.exe storageattach "<uuid|vmname>" --storagectl IDE --port 0 --device 0 --medium "none"


    let output = Command::new(manage::get_cmd())
        .args(["modifyvm", name, "--nic1", "nat"])
        .output()?;
    io::stdout().write_all(&output.stdout)?;

    let output = Command::new(manage::get_cmd())
        .args(["modifyvm", name, "--natpf1", "ssh,tcp,127.0.0.1,2201,,22"])
        .output()?;
    io::stdout().write_all(&output.stdout)?;

    Ok(())
}

async fn get_ova() -> Result<PathBuf> {
    let path = BOX_PATH.join("ova");
    std::fs::create_dir_all(&path)?;

    let ova_path = path.join("ubuntu-22_04.ova");
    if !ova_path.exists() {
        download_file(&Client::new(), UBUNTU, &ova_path).await?;
    }

    Ok(ova_path)
}

async fn make_init(path: &Path) -> Result<()> {   

    let mut file = OpenOptions::new().create(true).write(true).truncate(true).open(&path).await?;
    file.write_all(INIT_ISO).await?;

    Ok(())
}

async fn download_file(client: &Client, url: &str, path: &Path) -> Result<()> {
    // Reqwest setup
    let res = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Failed to GET from '{}'", &url))?;
    let total_size = res
        .content_length()
        .with_context(|| format!("Failed to get content length from '{}'", &url))?;

    // Indicatif setup
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .progress_chars("#>-"));
    //pb.set_message(&format!("Downloading {}", url));

    // download chunks
    let mut file =
        std::fs::File::create(path).with_context(|| (format!("Failed to create file '{:?}'", path)))?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.context("Error while downloading file")?;
        file.write_all(&chunk)
            .context("Error while writing to file")?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(&format!("Downloaded {} to {:?}", url, path));
    return Ok(());
}
