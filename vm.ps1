

# Change default pk with user pk


$global:ifname = "vboxnet0"

class OVA {

    static [void] Import( [string]$Name) {
    
        $ova = "$global:HOME/.xtec/xtec.ova"
        if (-not(Test-Path $ova -PathType Leaf)) {
            New-Item -Path "$global:HOME/.xtec" -ItemType Directory
            Write-Host("Downloading xtec.ova from Google Drive")
            DriveDownload -GoogleFileId "1UxNLsSvv7eo-M6MmAgadn7m14wEvrMmZ" -Destination $ova
        }

        vboxmanage import $ova --vsys 0 --vmname $Name
    }
}

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
            [OVA]::Import($this.Name)
        }


        vboxmanage modifyvm $this.Name --nic1 nat
        vboxmanage modifyvm $this.Name --natpf1 delete ssh
        vboxmanage modifyvm $this.Name --natpf1 "ssh,tcp,127.0.0.1,$($this.SSH),,22"
    
        #vboxmanage modifyvm $this.Name --nic2 hostonly --hostonlyadapter2 $global:ifname


        Write-Host(vboxmanage startvm $this.Name --type headless)

        # set-hostname
        # apply netplan
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

class SSH {

    hidden static [String] $key = "$global:HOME/.ssh/id_ed25519_xtec" 

    static [void] Config() {
    
        $file = [SSH]::key

        if (-not(Test-Path $file -PathType Leaf)) {
            New-Item -Path $file -ItemType File -Force
            
            # TODO Linux Set-Content $file -NoNewline  // and  permissions

            Set-Content $file "-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW
QyNTUxOQAAACBBeYoYAKtcIYloqqAe21RQoxtP/Zs1GzIN0uIz35mt2AAAAJB6j5yveo+c
rwAAAAtzc2gtZWQyNTUxOQAAACBBeYoYAKtcIYloqqAe21RQoxtP/Zs1GzIN0uIz35mt2A
AAAEC5SPSQvW7DimyU4MYx6SQCVAGXWCNNWmXMGEorEdt150F5ihgAq1whiWiqoB7bVFCj
G0/9mzUbMg3S4jPfma3YAAAABmFsdW1uZQECAwQFBgc=
-----END OPENSSH PRIVATE KEY-----"
        }
    }

    static [void] Connect([String] $id) {

        if(!$id) {
            $id = "1"
        }
        
        $file = [SSH]::key

        ssh -p 220$id -i $file  -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no alumne@127.0.0.1
    }
}



##### MAIN #####

if ( $env:OS -eq 'Windows_NT') {
    if ($null -eq (get-command VBoxManage.exe -errorAction silentlyContinue) ) {
        $env:path = "C:\Program Files\Oracle\VirtualBox;$env:path"
    }
}


[SSH]::Config()


# TODO check exists
#vboxmanage list hostonlyifs
# VBoxManage hostonlyif create
#vboxmanage hostonlyif ipconfig $ifname --ip 192.168.56.1 --netmask 255.255.255.0
#vboxmanage dhcpserver modify --ifname $ifname --disable



# vboxmanage list vms
$global:vms = New-Object Collections.Generic.List[VM]
foreach ($i in 1..1) {
    $vm = [VM]::new();
    $vm.Name = "xtec-$i"
    $vm.SSH = "220$i"
    $vms.Add($vm)
}


function BoxStart() {
    #$vms | Foreach-Object -ThrottleLimit 3 -Parallel { $_}
    foreach ($vm in $vms) {
        $vm.Start()
    }
}

function BoxStop() {
    # TODO list all xtec machines and stop
    foreach ($vm in $vms) {
        $vm.Stop()
    }
}


$cmd = $args[0]
switch ($cmd) {
    ssh {
        [SSH]::connect($args[1])
    }

    start {
        BoxStart
    }
    status {
        $vms = vboxmanage list vms
        foreach ($vm in $vms) {
            Write-Host $vm
        }
    }
    stop { 
        BoxStop 
    }
    Default {
        Write-Host("Usage: vm.ps1 start | stop | ssh | status")
    }
}









