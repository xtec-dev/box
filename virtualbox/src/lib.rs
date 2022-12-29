use anyhow::{bail, Context, Result};
use once_cell::sync::Lazy;
use std::collections::HashMap;

use std::fmt::Display;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use regex::Regex;

mod coreos;
mod manage;
//#[cfg(windows)]
//mod mscom;
mod ova;
mod ssh;
mod ubuntu;

// https://www.virtualbox.org/manual/ch08.html
// https://www.youtube.com/watch?v=eS_7WOnP2rs

static BOX_PATH: Lazy<PathBuf> = Lazy::new(|| home::home_dir().expect("Home dir").join(".box"));

pub async fn create(name: &str, image: Image) -> Result<()> {
    match image {
        Image::CoreOS => coreos::create(name).await,
        Image::Ubuntu => ubuntu::create(name).await,
    }
}

pub fn list_vms() -> Result<Vec<Machine>> {
    // TODO remove depenency
    let list = vboxhelper::get_vm_list()?;
    let vms: Vec<Machine> = list
        .iter()
        .map(|(name, _)| Machine { name: name.clone() })
        .collect();
    Ok(vms)
}

pub enum Image {
    CoreOS,
    Ubuntu,
}

#[derive(Clone)]
pub struct Machine {
    pub name: String,
}

impl Display for Machine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

impl Machine {
    pub fn new(name: String) -> Machine {
        Machine { name }
    }

    pub fn info(&self) -> Result<MachineInfo> {
        let mut cmd = Command::new(manage::get_cmd());
        cmd.args(["showvminfo", &self.name, "--machinereadable"]);

        let output = cmd
            .output()
            .with_context(|| format!("{}: getting info:", self))?;

        if output.status.success() {
            let configuration = std::str::from_utf8(&output.stdout)?;
            let info = MachineInfo(String::from(configuration));
            Ok(info)
        } else {
            let error = std::str::from_utf8(&output.stderr)?;
            if error.contains("VBOX_E_OBJECT_NOT_FOUND") {
                bail!("{} not found", self.name)
            } else {
                bail!(String::from(error))
            }
        }
    }

    pub async fn delete(&self) -> Result<()> {
        self.stop().await?;

        println!("{}: delete", self);
        let mut cmd = Command::new(manage::get_cmd());
        cmd.args(["unregistervm", &self.name, "--delete"]);

        let output = cmd.output()?;
        io::stdout().write_all(&output.stdout)?;

        if output.status.success() {
            Ok(())
        } else {
            let msg = String::from_utf8(output.stderr)?;
            bail!(format!("delete:{:?}", msg))
        }
    }

    pub async fn ssh(&self) -> Result<()> {
        self.start().await?;
        let port = self.info()?.ssh_port()?;
        ssh::connect(port).await
    }

    pub async fn start(&self) -> Result<()> {
        let state = self.info()?.state()?;
        if state == State::Running || state == State::Starting {
            return Ok(());
        }

        let mut cmd = Command::new(manage::get_cmd());
        cmd.args(["startvm", &self.name, "--type", "headless"]);
        let output = cmd.output()?;
        io::stdout().write_all(&output.stdout)?;

        if output.status.success() {
            Ok(())
        } else {
            let msg = String::from_utf8(output.stderr)?;
            bail!(format!("start:{:?}", msg))
        }
        // hostnamectl set-hostname --static xxx
    }

    pub async fn stop(&self) -> Result<()> {
        let mut state = self.info()?.state()?;
        if state == State::PowerOff {
            return Ok(());
        }

        print!("{}: stopping ", self);

        while state != State::PowerOff {
            if state != State::Stopping {
                let mut cmd = Command::new(manage::get_cmd());
                cmd.args(["controlvm", &self.name, "acpipowerbutton"]);
                let output = cmd.output()?;
                io::stdout().write_all(&output.stdout)?;

                if !output.status.success() {
                    let msg = String::from_utf8(output.stderr)?;
                    bail!(format!("stop:{:?}", msg))
                };
            }

            tokio::time::sleep(Duration::from_millis(300)).await;
            print!(".");
            state = self.info()?.state()?;
        }
        println!("");

        Ok(())
    }
}

pub struct MachineInfo(String);

//https://www.virtualbox.org/manual/ch08.html#vboxmanage-showvminfo
impl MachineInfo {
    // TODO use pest https://medium.com/code-zen/learn-to-build-a-parser-in-rust-for-fun-and-profit-e22ca0e0ce4c
    pub fn map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();

        // Capture foo="bar" -> foo=bar
        let re1 = Regex::new(r#"^(?P<key>[^"=]+)="(?P<val>[^"=]*)"$"#).unwrap();
        // Capture "foo"="bar" -> foo=bar
        let re2 = Regex::new(r#"^"(?P<key>[^"=]+)"="(?P<val>[^"=]*)"$"#).unwrap();
        // foo=bar -> foo=bar
        let re3 = Regex::new(r#"^(?P<key>[^"=]+)=(?P<val>[^"=]*)$"#).unwrap();

        for line in self.0.split("\n") {
            if line.len() == 0 {
                continue;
            }

            let line = line.trim_end();
            let cap = if let Some(cap) = re1.captures(&line) {
                Some(cap)
            } else if let Some(cap) = re2.captures(&line) {
                Some(cap)
            } else if let Some(cap) = re3.captures(&line) {
                Some(cap)
            } else {
                None
            };

            if let Some(cap) = cap {
                map.insert(cap[1].to_string(), cap[2].to_string());
            }
        }
        map
    }

    pub fn ssh_port(&self) -> Result<u16> {
        // Forwarding(1)="ssh,tcp,127.0.0.1,2206,,22"
        let regex = Regex::new(r#"^Forwarding.*="ssh.*,(\d*),,22"#).unwrap();

        for line in self.0.split("\n") {
            if line.len() == 0 {
                continue;
            }

            if let Some(caps) = regex.captures(line) {
                let port = caps[1].parse::<u16>()?;
                return Ok(port);
            }
        }

        bail!("ssh forward port not found")
    }

    pub fn state(&self) -> Result<State> {
        // VMState="poweroff"
        let regex = Regex::new(r#"^VMState="(\w*)""#).unwrap();


        for line in self.0.split("\n") {
            if line.len() == 0 {
                continue;
            }

            //println!("{}",line);

            if let Some(caps) = regex.captures(line) {
                let state = caps[1].parse::<String>()?;
                let state = match state.as_str() {
                    "poweroff" => State::PowerOff,
                    "starting" => State::Starting,
                    "running" => State::Running,
                    "paused" => State::Paused,
                    "stopping" => State::Stopping,
                    _ => bail!("unknown state {}", state),
                };
                return Ok(state);
            }
        }

        bail!("VMState property not found")
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum State {
    Unknown, // TODO remove
    PowerOff,
    Starting,
    Running,
    Paused,
    Stopping,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            State::Unknown => write!(f, "unknown"),
            State::PowerOff => write!(f, "poweroff"),
            State::Starting => write!(f, "starting"),
            State::Running => write!(f, "running"),
            State::Paused => write!(f, "paused"),
            State::Stopping => write!(f, "stopping"),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn info_ssh_port() -> Result<()> {
        let info = MachineInfo(String::from(
            r#"name="box"
nic1="nat"
nictype1="82540EM"
Forwarding(0)="http,tcp,127.0.0.1,8080,,80"
Forwarding(1)="ssh,tcp,127.0.0.1,2206,,22"
"#,
        ));

        assert_eq!(2206, info.ssh_port()?);
        Ok(())
    }

    #[test]
    fn info_map() {
        /* error
            r#" VBoxManage: error: Could not find a registered machine named 'xtec-4'
        VBoxManage: error: Details: code VBOX_E_OBJECT_NOT_FOUND (0x80bb0001), component VirtualBoxWrap, interface IVirtualBox, callee nsISupports
        VBoxManage: error: Context: "FindMachine(Bstr(VMNameOrUuid).raw(), machine.asOutParam())" at line 3138 of file VBoxManageInfo.cpp"#,
        ).is_none());*/

        let _info = MachineInfo(String::from(
            r#"name="xtec-1"
        Encryption:     disabled
        groups="/"
        ostype="Ubuntu (64-bit)"
        UUID="02fd76af-2bae-4c98-add1-4d802a2f77fa"
        CfgFile="/home/david/.xtec/xtec-1/xtec-1.vbox"
        SnapFldr="/home/david/.xtec/xtec-1/Snapshots"
        LogFldr="/home/david/.xtec/xtec-1/Logs"
        hardwareuuid="02fd76af-2bae-4c98-add1-4d802a2f77fa"
        memory=6144
        pagefusion="off"
        vram=4
        cpuexecutioncap=100
        hpet="off"
        cpu-profile="host"
        chipset="piix3"
        firmware="BIOS"
        cpus=4
        pae="on"
        longmode="on"
        triplefaultreset="off"
        apic="on"
        x2apic="on"
        nested-hw-virt="off"
        cpuid-portability-level=0
        bootmenu="messageandmenu"
        boot1="disk"
        boot2="dvd"
        boot3="none"
        boot4="none"
        acpi="on"
        ioapic="on"
        biosapic="apic"
        biossystemtimeoffset=0
        BIOS NVRAM File="/home/david/.xtec/xtec-1/xtec-1.nvram"
        rtcuseutc="on"
        hwvirtex="on"
        nestedpaging="on"
        largepages="off"
        vtxvpid="on"
        vtxux="on"
        virtvmsavevmload="on"
        iommu="none"
        paravirtprovider="default"
        effparavirtprovider="kvm"
        VMState="poweroff"
        VMStateChangeTime="2022-12-17T09:55:20.000000000"
        graphicscontroller="vboxvga"
        monitorcount=1
        accelerate3d="off"
        accelerate2dvideo="off"
        teleporterenabled="off"
        teleporterport=0
        teleporteraddress=""
        teleporterpassword=""
        tracing-enabled="off"
        tracing-allow-vm-access="off"
        tracing-config=""
        autostart-enabled="off"
        autostart-delay=0
        defaultfrontend=""
        vmprocpriority="default"
        storagecontrollername0="IDE Controller"
        storagecontrollertype0="PIIX4"
        storagecontrollerinstance0="0"
        storagecontrollermaxportcount0="2"
        storagecontrollerportcount0="2"
        storagecontrollerbootable0="on"
        "IDE Controller-0-0"="/home/david/.xtec/xtec-1/xtec-disk001.vmdk"
        "IDE Controller-ImageUUID-0-0"="74a4d9fd-e083-4891-ab51-9c955308a933"
        "IDE Controller-nonrotational-0-0"="off"
        "IDE Controller-discard-0-0"="off"
        "IDE Controller-0-1"="none"
        "IDE Controller-1-0"="none"
        "IDE Controller-1-1"="none"
        natnet1="nat"
        macaddress1="0800271A4565"
        cableconnected1="on"
        nic1="nat"
        nictype1="82540EM"
        nicspeed1="0"
        mtu="0"
        sockSnd="64"
        sockRcv="64"
        tcpWndSnd="64"
        tcpWndRcv="64"
        Forwarding(0)="ssh,tcp,127.0.0.1,2201,,22"
        hostonlyadapter2="VirtualBox Host-Only Ethernet Adapter"
        macaddress2="080027AE1F98"
        cableconnected2="on"
        nic2="hostonly"
        nictype2="82540EM"
        nicspeed2="0"
        nic3="none"
        nic4="none"
        nic5="none"
        nic6="none"
        nic7="none"
        nic8="none"
        hidpointing="ps2mouse"
        hidkeyboard="ps2kbd"
        uart1="off"
        uart2="off"
        uart3="off"
        uart4="off"
        lpt1="off"
        lpt2="off"
        audio="none"
        audio_out="off"
        audio_in="off"
        clipboard="disabled"
        draganddrop="disabled"
        vrde="on"
        vrdeport=-1
        vrdeports="5983"
        vrdeaddress="0.0.0.0"
        vrdeauthtype="null"
        vrdemulticon="off"
        vrdereusecon="off"
        vrdevideochannel="off"
        usb="off"
        ehci="off"
        xhci="off"
        recording_enabled="off"
        recording_screens=1
         rec_screen0
        rec_screen_enabled="off"
        rec_screen_id=0
        rec_screen_video_enabled="on"
        rec_screen_audio_enabled="off"
        rec_screen_dest="File"
        rec_screen_dest_filename="/home/david/.xtec/xtec-1/xtec-1-screen0.webm"
        rec_screen_opts="vc_enabled=true,ac_enabled=true,ac_profile=med"
        rec_screen_video_res_xy="1024x768"
        rec_screen_video_rate_kbps=512
        rec_screen_video_fps=25
        GuestMemoryBalloon=0
        "#,
        ));
    }
}
