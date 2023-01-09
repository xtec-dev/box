use std::io::Write;
use std::process::Command;

use anyhow::{bail, Context, Result};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use crate::manage;

static HOST_MUTEX: Lazy<Mutex<u16>> = Lazy::new(|| Mutex::new(0));

/*

vboxmanage showvminfo core | grep "NIC 1:" | awk '{print tolower($4)}' | sed 's/.\{2\}/&:/g' | sed 's/.\{2\}$//'

arp -a | grep 08:00:27:a5:db:a3 | awk '{print $2}' | tail -c +2 | head -c -2
*/

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

pub fn get_hostonly(name: &str) -> Result<String> {
    let host = get_host(name)?;
    match host {
        None => bail!("{}: no host only nic set:", name),
        Some(host) => Ok(format!("192.168.56.{}", host)),
    }
}

// ip  -o -4 addr
// IP address for the host  192.168.56.1
// DHCP-Server Range        192.168.56.101 - 192.168.56.254
// The IP range limiting the IP addresses that will be provided to the guest systems 192.168.56.2 - 192.168.56.100
pub async fn set_hostonly(name: &str) -> Result<()> {
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

    set_host(name).await
}

const KEY: &str = "hostonly";

pub fn get_host(name: &str) -> Result<Option<u8>> {
    let output = Command::new(manage::get_cmd())
        .args(["getextradata", name, KEY])
        .output()?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        let mut result = result.split(':');
        if let (Some(_), Some(host)) = (result.next(), result.next()) {
            let host: u8 = host
                .trim()
                .parse::<u8>()
                .with_context(|| format!("parsing {}", host))?;
            Ok(Some(host))
        } else {
            Ok(None)
        }
    } else {
        // let err = String::from_utf8_lossy(&output.stderr);
        bail!("todo: getextradata error");
    }
}

async fn set_host(name: &str) -> Result<()> {
    if get_host(name)?.is_some() {
        return Ok(());
    };

    let _lock = HOST_MUTEX.lock().await;

    let mut hosts = Vec::new();
    for vm in crate::list_vms()? {
        if let Some(host) = get_host(&vm.name)? {
            hosts.push(host);
        }
    }
    hosts.sort();

    let mut host: u8 = 15;
    for port in hosts {
        if port == host {
            host += 1;
        } else {
            break;
        }
    }

    let output = Command::new(manage::get_cmd())
        .args(["setextradata", name, KEY, &host.to_string()])
        .output()?;
    std::io::stdout().write_all(&output.stdout)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use anyhow::Result;

    use super::*;

    #[test]
    fn test_host() -> Result<()> {
        let host = get_host("xtec-1")?;
        assert_eq!(Some(2), host);

        Ok(())
    }
}
