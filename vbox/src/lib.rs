use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::{cmp::min, path::Path};

use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;

pub async fn start(_id: u16) -> Result<Machine> {
    let vm = Machine {
        name: String::from("xtec-1"),
    };

    let _vms = list_vms();

    if false {
        import(&vm).await?;
    }
    Ok(vm)
}

async fn import(vm: &Machine) -> Result<()> {
    let mut path = home::home_dir().expect("Home dir");
    path = path.join(".xtec");
    std::fs::create_dir_all(&path)?;

    let ova_path = path.join("xtec.ova");
    if !ova_path.exists() {
        download_file(&Client::new(), "https://xtec.optersoft.com/xtec.ova", &path)
            .await
            .unwrap();
    }

    println!("box: importing virtual machine {}", vm.name);
    let output = Command::new("vboxmanage")
        .arg("import")
        .arg(ova_path)
        .args(["--vsys", "0", "--vmname", &vm.name, "--basefolder"])
        .arg(path)
        .spawn()?;
    println!("{:?}", output);

    //vboxmanage import ([Box]::ova) --vsys 0 --vmname $Name --basefolder ([Box]::home)

    Ok(())
}

pub fn list_vms() -> Result<Vec<Machine>> {
    let vms = Command::new("vboxmanage").arg("list").arg("vms").output()?;

    println!("{:#?}", vms);

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

pub struct Machine {
    name: String,
}

impl Machine {
    pub fn _start() -> Result<()> {
        Command::new("vboxmanage").arg("list").arg("vms").spawn()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    // "\"xtec-1\" {9faa6303-ad35-4b75-b678-912ceb3c2bce}\n",

    use std::collections::HashSet;

    use vboxhelper;

    fn test() {
        // Get list of all known virtual machines in system
        let lst = vboxhelper::get_vm_list().expect("Unable to get VM list");

        // Get a HashSet containing all known _running_ virtual machines
        let running = {
            let mut set = HashSet::new();
            for (_, uuid) in vboxhelper::get_running_vms_list().expect("Unable to get VM list") {
                set.insert(uuid);
            }

            set
        };

        // Find the longest virtual machine name, to make make output visually
        // stunning.
        let mut max_len = 0;
        for (nm, _) in &lst {
            if nm.len() > max_len {
                max_len = nm.len();
            }
        }

        // Display a list of all virtual machines, and marking the running ones.
        for (nm, uuid) in &lst {
            let runstate = if running.contains(&uuid) {
                " [running]"
            } else {
                ""
            };

            println!("{:width$}  {}{}", nm, uuid, runstate, width = max_len);
        }
    }
}
