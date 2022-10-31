
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

    [void] Start() {
        Write-Host(vboxmanage startvm $this.Name --type headless)
    }

    # TODO wait
    [void] Stop() {
        Write-Host(vboxmanage controlvm $this.Name acpipowerbutton)
        #echo "Waiting for machine $MACHINE to poweroff..."
        #until $(VBoxManage showvminfo --machinereadable $MACHINE | grep -q ^VMState=.poweroff.)
        #do
        #  sleep 1
        #done
    }
}

$vms = New-Object Collections.Generic.List[VM]
foreach ($i in 1..2) {
    $vm = [VM]::new();
    $vm.Name = "xtec-$i"
    $vm.SSH = "220$i"
    $vms.Add($vm)
}

# vboxmanage list vms


$ifname = "vboxnet0"
# TODO check exists
#vboxmanage list hostonlyifs
# VBoxManage hostonlyif create
vboxmanage hostonlyif ipconfig $ifname --ip 192.168.56.1 --netmask 255.255.255.0
vboxmanage dhcpserver modify --ifname $ifname --disable

foreach ($vm in $vms) {

    $vm.Stop()
    
    vboxmanage modifyvm $vm.Name --nic1 nat
    vboxmanage modifyvm $vm.Name --natpf1 delete ssh
    vboxmanage modifyvm $vm.Name --natpf1 "ssh,tcp,127.0.0.1,$($vm.SSH),,22"
    
    vboxmanage modifyvm $vm.Name --nic2 hostonly --hostonlyadapter2 $ifname
    $vm.Start()
}

#$vms | Foreach-Object -ThrottleLimit 3 -Parallel { $_}








