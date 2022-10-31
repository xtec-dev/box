
if ( $IsWindows) {
    if ($null -eq (get-command VBoxManage.exe -errorAction silentlyContinue) ) {
        $env:path = "C:\Program Files\Oracle\VirtualBox;$env:path"
    }
}

#Write-Host(VBoxManage startvm xtec-1 --type headless)
#VBoxManage showvminfo $vmName --machinereadable 


class VM {
    [string]$Name
    [String]$SSH
}

$vms = New-Object Collections.Generic.List[VM]
foreach ($i in 1..2) {
    $vm = [VM]::new();
    $vm.Name = "xtec-$i"
    $vm.SSH = "220$i"
    $vms.Add($vm)
}

# vboxmanage list vms

# https://docs.oracle.com/en/virtualization/virtualbox/6.0/user/vboxmanage-hostonlyif.html
# https://docs.oracle.com/en/virtualization/virtualbox/6.0/user/network_hostonly.html
# https://gist.github.com/magnetikonline/46a483cc8c9d0451074642f860d0cac1

# TODO check exists

#vboxmanage list hostonlyifs
# VBoxManage hostonlyif create

$ifname = "vboxnet0"
vboxmanage hostonlyif ipconfig $ifname --ip 192.168.56.1 --netmask 255.255.255.0
vboxmanage dhcpserver modify --ifname $ifname --disable

foreach ($vm in $vms) {
    vboxmanage modifyvm $vm.Name --nic1 nat
    vboxmanage modifyvm $vm.Name --nic2 hostonly --hostonlyadapter2 $ifname
}

# https://www.how2shout.com/how-to/vboxmanage-command-not-found-in-windows-cmd-or-powershell.html






