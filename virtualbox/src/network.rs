use std::io::Write;
use std::process::Command;

use anyhow::Result;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use crate::manage;

static FORWARD_MUTEX: Lazy<Mutex<u16>> = Lazy::new(|| Mutex::new(0));

/*

$adapter = "VirtualBox Host-Only Ethernet Adapter"
        #Linux "vboxnet0"

        #vboxmanage hostonlyif ipconfig $ifname --ip 192.168.56.1 --netmask 255.255.255.0
        #vboxmanage dhcpserver modify --ifname $ifname --disable

        # TODO
        <#$list = vboxmanage list hostonlyifs
        $name = $list[0] | Select-String '^Name:\s+(\w+)'
        if ($list -notlike "*$adapter*") {
            Write-Host("box: host-only not found: $adapter")
            exit
        }#>
*/

// https://www.virtualbox.org/manual/ch08.html#vboxmanage-dhcpserver

// ip  -o -4 addr

pub fn set_hostonly(name: &str) -> Result<()> {
    /*
        let mut cmd = Command::new(manage::get_cmd());
        cmd.args(["list", "--hostonlyifs"]);
        let ouput = cmd.output()?;
    */
    #[cfg(target_os = "windows")]
    let adapter = "VirtualBox Host-Only Ethernet Adapter";

    #[cfg(target_os = "linux")]
    let adapter = "vboxnet0";

    let output = Command::new(manage::get_cmd())
        .args([
            "modifyvm",
            name,
            "--nic2",
            "hostonly",
            "--hostonlyadapter2",
            &adapter,
        ])
        .output()?;
    std::io::stdout().write_all(&output.stdout)?;

    Ok(())
}

pub async fn set_port_forward(name: &str) -> Result<u16> {
    let _lock = FORWARD_MUTEX.lock().await;

    let mut ports = Vec::new();
    for vm in crate::list_vms()? {
        if let Ok(port) = vm.info()?.ssh_port() {
            ports.push(port);
        }
    }
    ports.sort();

    let mut ssh_port: u16 = 2201;
    for port in ports {
        if port == ssh_port {
            ssh_port += 1;
        } else {
            break;
        }
    }

    let rule = format!("ssh,tcp,127.0.0.1,{},,22", ssh_port);

    let output = Command::new(manage::get_cmd())
        .args(["modifyvm", name, "--natpf1", &rule])
        .output()?;
    std::io::stdout().write_all(&output.stdout)?;
    Ok(ssh_port)
}
