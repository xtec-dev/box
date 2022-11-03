

# TODO clone 2,3
# change ova with default ssh public key
# reduce ova size, rm vagrant folder and mount

# TODO   vm start x | stop 

$ova = "xtec.ova"

class VM {
    [string]$Name
    [String]$SSH

    [void] Delete() {
        $this.Stop()
        Write-Host(vboxmanage unregister $this.name --delete)
        #rmdir -recurse $vmPath
        #rmdir -recurse $sharedFolder
    }

    [void] Start() {

        $vms = vboxmanage list vms
        if ($vms -like "*$($this.Name)*") {
            $this.Stop()
        }
        else {
            Write-Host("Could not find a registered machine named '$($this.Name)'")
            vboxmanage import xtec.ova --vsys 0 --vmname $this.Name
        }
        

        exit

        Write-Host(vboxmanage startvm $this.Name --type headless)
    }

    [void] Stop() {
        # TODO State()

        $info = VBoxManage showvminfo --machinereadable $this.Name
        while (-not($info -like 'VMState="poweroff"')) {
            vboxmanage controlvm $this.Name acpipowerbutton
            Write-Host("Waiting for machine $($this.Name) to poweroff...")
            Start-Sleep -Seconds 2
            $info = VBoxManage showvminfo --machinereadable $this.Name
        }
    }
}

function DriveDownload {
    param(
        [string]$GoogleFileId,
        [string]$Destination)

    # set protocol to tls version 1.2
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

    $source = "https://drive.google.com/uc?export=download&confirm=t&id=$GoogleFileId"

    if ( $env:OS -eq 'Windows_NT') {
        Start-BitsTransfer -Source $source -Destination $Destination
    }
    else {
        Write-Host("TODO wget in linux")
    }
}



##### MAIN #####

if ( $env:OS -eq 'Windows_NT') {
    if ($null -eq (get-command VBoxManage.exe -errorAction silentlyContinue) ) {
        $env:path = "C:\Program Files\Oracle\VirtualBox;$env:path"
    }
}



if (-not(Test-Path $ova -PathType Leaf)) {
    Write-Host("Downloading xtec.ova from Google Drive")
    DriveDownload -GoogleFileId "1UxNLsSvv7eo-M6MmAgadn7m14wEvrMmZ" -Destination "xtec.ova"
}


# VBoxManage clonevm $vmName --name=$vmNameClone --register

# vboxmanage list vms
$vms = New-Object Collections.Generic.List[VM]
foreach ($i in 2..2) {
    $vm = [VM]::new();
    $vm.Name = "xtec-$i"
    $vm.SSH = "220$i"
    $vms.Add($vm)
}


$ifname = "vboxnet0"
# TODO check exists
#vboxmanage list hostonlyifs
# VBoxManage hostonlyif create
vboxmanage hostonlyif ipconfig $ifname --ip 192.168.56.1 --netmask 255.255.255.0
vboxmanage dhcpserver modify --ifname $ifname --disable

foreach ($vm in $vms) {

    <#
    $vm.Stop()
    
    vboxmanage modifyvm $vm.Name --nic1 nat
    vboxmanage modifyvm $vm.Name --natpf1 delete ssh
    vboxmanage modifyvm $vm.Name --natpf1 "ssh,tcp,127.0.0.1,$($vm.SSH),,22"
    
    vboxmanage modifyvm $vm.Name --nic2 hostonly --hostonlyadapter2 $ifname
    #>
    $vm.Start()
}

#$vms | Foreach-Object -ThrottleLimit 3 -Parallel { $_}








