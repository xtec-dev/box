use anyhow::{bail, Context, Result};
use once_cell::sync::Lazy;
use std::collections::HashMap;

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

use regex::Regex;

mod manage;
//#[cfg(windows)]
//mod manager;
mod ova;

// https://www.virtualbox.org/manual/ch08.html
//# ssh 2201 -i ~/.ssh/id_ed25519_box -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no alumne@127.0.0.1

// $ssh = "-p $($vm.SSH) -i $([SSH]::key) -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no alumne@127.0.0.1"

static BOX_PATH: Lazy<PathBuf> = Lazy::new(|| home::home_dir().expect("Home dir").join(".box"));

pub fn list_vms() -> Result<Vec<Machine>> {
    let list = vboxhelper::get_vm_list()?;
    let vms: Vec<Machine> = list
        .iter()
        .map(|(name, _)| Machine { name: name.clone() })
        .collect();
    Ok(vms)
}

#[derive(Clone)]
pub struct Machine {
    pub name: String,
}

impl Machine {
    pub fn new(name: String) -> Machine {
        Machine { name }
    }

    pub fn info(&self) -> Result<Option<MachineInfo>> {
        let mut cmd = Command::new(manage::get_cmd());
        cmd.arg("showvminfo");
        cmd.arg(&self.name);
        cmd.arg("--machinereadable");

        let output = cmd
            .output()
            .with_context(|| format!("{}: getting info:", self.name))?;

        if output.status.success() {
            let configuration = std::str::from_utf8(&output.stdout)?;
            let info = MachineInfo::parse(configuration);
            Ok(Some(info))
        } else {
            let error = std::str::from_utf8(&output.stderr)?;
            if error.contains("VBOX_E_OBJECT_NOT_FOUND") {
                Ok(None)
            } else {
                bail!(String::from(error))
            }
        }
    }

    pub async fn delete(&self) -> Result<()> {
        self.stop().await?;

        println!("{}: deleting", self.name);
        let mut cmd = Command::new(manage::get_cmd());
        cmd.arg("unregistervm");
        cmd.arg(&self.name);
        cmd.arg("--delete");

        //println!("Starting vm {}", self.name);

        let output = cmd.output()?;
        io::stdout().write_all(&output.stdout)?;

        if output.status.success() {
            Ok(())
        } else {
            let msg = String::from_utf8(output.stderr)?;
            bail!(format!("delete:{:?}", msg))
        }
    }

    pub async fn start(&self) -> Result<()> {
        match self.info()? {
            None => ova::import(&self.name).await?,
            Some(info) => {
                let _state = info.get_state()?;
                //println!("state {}", _state);
            }
        };

        let mut cmd = Command::new(manage::get_cmd());
        cmd.arg("startvm");
        cmd.arg(&self.name);
        cmd.arg("--type");
        cmd.arg("headless");

        //println!("Starting vm {}", self.name);

        let output = cmd.output()?;
        io::stdout().write_all(&output.stdout)?;

        if output.status.success() {
            Ok(())
        } else {
            let msg = String::from_utf8(output.stderr)?;
            bail!(format!("start:{:?}", msg))
        }
    }

    pub async fn stop(&self) -> Result<()> {
        match self.info()? {
            None => return Ok(()),
            Some(info) => {
                let state = info.get_state()?;
                if state == MachineState::PowerOff || state == MachineState::Stopping {
                    return Ok(());
                }
            }
        }

        let mut cmd = Command::new(manage::get_cmd());
        cmd.arg("controlvm");
        cmd.arg(&self.name);
        cmd.arg("acpipowerbutton");

        //println!("Starting vm {}", self.name);

        let output = cmd.output()?;
        io::stdout().write_all(&output.stdout)?;

        if output.status.success() {
            Ok(())
        } else {
            let msg = String::from_utf8(output.stderr)?;
            bail!(format!("stop:{:?}", msg))
        }
    }
}

pub struct MachineInfo(HashMap<String, String>);

//https://www.virtualbox.org/manual/ch08.html#vboxmanage-showvminfo
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

    // TODO use pest https://medium.com/code-zen/learn-to-build-a-parser-in-rust-for-fun-and-profit-e22ca0e0ce4c
    fn parse(str: &str) -> MachineInfo {
        let mut map = HashMap::new();

        // Capture foo="bar" -> foo=bar
        let re1 = Regex::new(r#"^(?P<key>[^"=]+)="(?P<val>[^"=]*)"$"#).unwrap();
        // Capture "foo"="bar" -> foo=bar
        let re2 = Regex::new(r#"^"(?P<key>[^"=]+)"="(?P<val>[^"=]*)"$"#).unwrap();
        // foo=bar -> foo=bar
        let re3 = Regex::new(r#"^(?P<key>[^"=]+)=(?P<val>[^"=]*)$"#).unwrap();

        for line in str.split("\n") {
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

        MachineInfo(map)
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

impl std::fmt::Display for MachineState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            MachineState::Unknown => write!(f, "unknown"),
            MachineState::PowerOff => write!(f, "poweroff"),
            MachineState::Starting => write!(f, "starting"),
            MachineState::Running => write!(f, "running"),
            MachineState::Paused => write!(f, "paused"),
            MachineState::Stopping => write!(f, "stopping"),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_machineinfo_parse() {
        /* error
            r#" VBoxManage: error: Could not find a registered machine named 'xtec-4'
        VBoxManage: error: Details: code VBOX_E_OBJECT_NOT_FOUND (0x80bb0001), component VirtualBoxWrap, interface IVirtualBox, callee nsISupports
        VBoxManage: error: Context: "FindMachine(Bstr(VMNameOrUuid).raw(), machine.asOutParam())" at line 3138 of file VBoxManageInfo.cpp"#,
        ).is_none());*/

        let _info = MachineInfo::parse(
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
        );
    }
}
