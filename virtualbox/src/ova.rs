use anyhow::{Context, Result};
use futures_util::StreamExt;
use reqwest::Client;
use std::io::Write;
use std::path::PathBuf;
use std::{cmp::min, path::Path};

use indicatif::{ProgressBar, ProgressStyle};

// TODO struct ova with hash

pub async fn get(name: &str, url: &str) -> Result<PathBuf> {
    let path = core::BOX_PATH.join("ova");
    std::fs::create_dir_all(&path)?;

    let name = format!("{}.ova", name);

    let ova_path = path.join(&name);
    if !ova_path.exists() {
        println!("ova: import {}", name);
        download_file(&Client::new(), &url, &ova_path).await?;
    }

    Ok(ova_path)
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
    let mut file = std::fs::File::create(path)
        .with_context(|| (format!("Failed to create file '{:?}'", path)))?;
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

/*
https://docs.fileformat.com/disc-and-media/ova/

An OVA (Open Virtual Appliance) file is an OVF directory saved as an archive using the .tar archiving format. It is a virtual appliance package file that contains files for distribution of software that runs on a virtual machine.

 */
fn _box() {}
