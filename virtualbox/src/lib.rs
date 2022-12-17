use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::{cmp::min, path::Path};
use vboxhelper::{RunContext, Shutdown, VmId};

use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use reqwest::Client;
use strutils::EmptyLine;

mod manage;
mod strutils;

pub fn list_vms() -> Result<Vec<Machine>> {
    let list = vboxhelper::get_vm_list()?;
    let vms: Vec<Machine> = list
        .iter()
        .map(|(name, _)| Machine { name: name.clone() })
        .collect();
    Ok(vms)
}

pub async fn stop(id: u16) -> Result<()> {
    let name = format!("xtec-{}", id);
    vboxhelper::controlvm::shutdown(&VmId::Name(name), Shutdown::AcpiPowerOff)?;
    Ok(())
}

async fn import(vm: &Machine) -> Result<()> {
    let mut path = home::home_dir().expect("Home dir");
    path = path.join(".xtec");
    std::fs::create_dir_all(&path)?;

    let ova_path = path.join("xtec.ova");
    if !ova_path.exists() {
        download_file(
            &Client::new(),
            "https://xtec.optersoft.com/xtec.ova",
            &ova_path,
        )
        .await?;
    }

    println!("box: importing virtual machine {}", vm.name);
    let output = Command::new(manage::get_cmd())
        .arg("import")
        .arg(ova_path)
        .args(["--vsys", "0", "--vmname", &vm.name, "--basefolder"])
        .arg(path)
        .spawn()?;
    println!("{:?}", output);

    //vboxmanage import ([Box]::ova) --vsys 0 --vmname $Name --basefolder ([Box]::home)

    Ok(())
}

// TODO check file exists
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
    pb.set_message(&format!("Downloading {}", url));

    // download chunks
    let mut file =
        File::create(path).with_context(|| (format!("Failed to create file '{:?}'", path)))?;
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

#[derive(Clone)]
pub struct Machine {
    pub name: String,
}

impl Machine {
    pub fn new(id: u16) -> Machine {
        let name = format!("xtec-{}", id);
        Machine { name }
    }

    pub fn _start() -> Result<()> {
        Ok(())
    }

    pub fn info(&self) -> Result<MachineInfo> {
        let mut cmd = Command::new(manage::get_cmd());
        cmd.arg("showvminfo");
        cmd.arg(&self.name);
        cmd.arg("--machinereadable");

        let output = cmd.output().expect("Failed to execute VBoxManage");

        let lines = strutils::buf_to_strlines(&output.stdout, EmptyLine::Ignore);

        let mut map = HashMap::new();

        // multiline
        //let re_ml1 =
        // Regex::new(r#"^(?P<key>[^"=]+)="(?P<val>[^"=]*)"$"#).unwrap();
        // let re_ml1 =
        // Regex::new(r#"^"(?P<key>[^"=]+)"="(?P<val>[^"=]*)"$"#).unwrap();

        // Capture foo="bar" -> foo=bar
        // This appears to be most common.
        let re1 = Regex::new(r#"^(?P<key>[^"=]+)="(?P<val>[^"=]*)"$"#).unwrap();

        // Capture "foo"="bar" -> foo=bar
        let re2 = Regex::new(r#"^"(?P<key>[^"=]+)"="(?P<val>[^"=]*)"$"#).unwrap();

        // foo=bar -> foo=bar
        let re3 = Regex::new(r#"^(?P<key>[^"=]+)=(?P<val>[^"=]*)$"#).unwrap();

        //let re = Regex::new(r#"^"?(?P<key>[^"=]+)"?="?(?P<val>[^"=]*)"?$"#).
        // unwrap();

        // ToDo: Handle multiline entires, like descriptions
        let mut lines = lines.iter();
        while let Some(line) = lines.next() {
            //println!("line: {}", line);

            let line = line.trim_end();
            let cap = if let Some(cap) = re1.captures(&line) {
                Some(cap)
            } else if let Some(cap) = re2.captures(&line) {
                Some(cap)
            } else if let Some(cap) = re3.captures(&line) {
                Some(cap)
            } else {
                //dbg!(format!("Ignored line: {}", line));
                None
            };

            if let Some(cap) = cap {
                map.insert(cap[1].to_string(), cap[2].to_string());
            }
        }
        let info = MachineInfo(map);
        Ok(info)
    }

    pub async fn start(&self) -> Result<()> {
        if list_vms()?
            .iter()
            .find(|&vm| vm.name == self.name)
            .is_none()
        {
            import(self).await?;
        }

        println!("Starting vm {}", self.name);
        vboxhelper::controlvm::start(
            &VmId::Name(self.name.clone()),
            RunContext::Headless(vboxhelper::Headless::Blocking),
        )?;

        //Command::new("vboxmanage").arg("list").arg("vms").spawn()?;

        Ok(())
    }
}

pub struct MachineInfo(HashMap<String, String>);

impl MachineInfo {
    pub fn get_state(&self) -> Result<MachineState> {
        let state = match self.0.get("VMState") {
            None => MachineState::Unknown,
            Some(s) => match s.as_ref() {
                "poweroff" => MachineState::PowerOff,
                "starting" => MachineState::Starting,
                "running" => MachineState::Running,
                "paused" => MachineState::Paused,
                "stopping" => MachineState::Stopping,
                _ => MachineState::Unknown,
            },
        };
        Ok(state)
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum MachineState {
    Unknown, // TODO remove
    PowerOff,
    Starting,
    Running,
    Paused,
    Stopping,
}

/*
                let state = match info.state {
                    vboxhelper::VmState::Unknown => "",
                    vboxhelper::VmState::PowerOff => "poweroff",
                    vboxhelper::VmState::Starting => "starting",
                    vboxhelper::VmState::Running => "running",
                    vboxhelper::VmState::Paused => "paused",
                    vboxhelper::VmState::Stopping => "stopping",
                };
*/

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
