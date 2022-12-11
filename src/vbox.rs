use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::{cmp::min, path::Path};

use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;

pub async fn start(_id: u16) -> Result<()> {
    import("xtec-1").await
}

async fn import(_name: &str) -> Result<()> {
    let mut path = home::home_dir().expect("Home dir");
    path = path.join(".xtec");
    std::fs::create_dir_all(&path)?;

    path = path.join("xtec.ova");
    if !path.exists() {
        download_file(&Client::new(), "https://xtec.optersoft.com/xtec.ova", &path)
            .await
            .unwrap();
    }

    let _vm = Machine {};

    Ok(())
}

pub struct Machine {}

impl Machine {
    pub fn _start() -> Result<()> {
        Command::new("vboxmanage").arg("list").arg("vms").spawn()?;
        Ok(())
    }
}

pub fn _list_vms() -> Result<Vec<Machine>> {
    let _list = Command::new("vboxmanage").arg("list").arg("vms").spawn()?;

    Ok(Vec::new())
}

async fn download_file(client: &Client, url: &str, path: &Path) -> Result<(), String> {
    // Reqwest setup
    let res = client
        .get(url)
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", &url)))?;
    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", &url))?;

    // Indicatif setup
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .progress_chars("#>-"));
    pb.set_message(&format!("Downloading {}", url));

    // download chunks
    let mut file = File::create(path).or(Err(format!("Failed to create file '{:?}'", path)))?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        file.write_all(&chunk)
            .or(Err(format!("Error while writing to file")))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(&format!("Downloaded {} to {:?}", url, path));
    return Ok(());
}
