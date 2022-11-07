
# TODO Change default pk with user pk

class Box {

    hidden static [String] $home = "$global:HOME/.xtec"
    hidden static [String] $ova = "$([Box]::home)/xtec.ova"

    static [String] Adapter() {

        $adapter = "VirtualBox Host-Only Ethernet Adapter"
        #Linux "vboxnet0"
        
        # TODO
        <#$list = vboxmanage list hostonlyifs
        $name = $list[0] | Select-String '^Name:\s+(\w+)'
        if ($list -notlike "*$adapter*") {
            Write-Host("box: host-only not found: $adapter")
            exit
        }#>

        return $adapter
    }


    static [void] Import( [string] $Name) {
    
        if (-not(Test-Path ([Box]::ova) -PathType Leaf)) {
            [Box]::Update()
        }

        Write-Host("box: importing virtual machine $Name")
        vboxmanage import ([Box]::ova) --vsys 0 --vmname $Name --basefolder ([Box]::home)
    }

    static [void] Update() {
        if (!(Test-Path -PathType Container ([Box]::home))) {
            New-Item -ItemType Directory -Path ([Box]::home) 
        }
        
        Write-Host("box: downloading xtec.ova")
        [Box]::Download("1UxNLsSvv7eo-M6MmAgadn7m14wEvrMmZ",([Box]::ova))
    }

    static [void] Download ( [string]$GoogleFileId, [string]$Destination) {
    
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

        [SSH]::Execute($this, "touch hello")

        $vms = vboxmanage list vms
        if ($vms -like "*$($this.Name)*") {
            $info = VBoxManage showvminfo --machinereadable $this.Name
            if ($info -like 'VMState="running"') {
                Write-Host("$($this.Name): machine is running")
                exit
            }
        }
        else {
            Write-Host("$($this.Name): machine it's not registered")
            [Box]::Import($this.Name)
        }


        vboxmanage modifyvm $this.Name --nic1 nat
        vboxmanage modifyvm $this.Name --natpf1 delete ssh
        vboxmanage modifyvm $this.Name --natpf1 "ssh,tcp,127.0.0.1,$($this.SSH),,22"

        $adapter = [Box]::Adapter()
        vboxmanage modifyvm $this.Name --nic2 hostonly --hostonlyadapter2 $adapter


        Write-Host("$($this.Name): startig machine ...")
        Write-Host(vboxmanage startvm $this.Name --type headless)

        # set-hostname
        # apply netplan
    }

    [void] Stop() {

        $vms = vboxmanage list vms
        if ($vms -notlike "*$($this.Name)*") {
            Write-Host("$($this.Name): machine not found.")
            exit
        }

        $info = VBoxManage showvminfo --machinereadable $this.Name
        while (-not($info -like 'VMState="poweroff"')) {
            vboxmanage controlvm $this.Name acpipowerbutton
            Write-Host("$($this.Name): waiting to poweroff machine ...")
            Start-Sleep -Seconds 2
            $info = VBoxManage showvminfo --machinereadable $this.Name
        }
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

    static [void] Connect([VM] $vm) {
        
        $ssh = "-p $($vm.SSH) -i $([SSH]::key) -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no alumne@127.0.0.1"

        Write-Host("ssh $ssh")
        Start-Process ssh $ssh

        # Linux 
        #ssh
    }

    static [void] Execute([VM] $vm,[String] $cmd) {

        $ssh = "-p $($vm.SSH) -i $([SSH]::key) -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no alumne@127.0.0.1 '$cmd'"

        Write-Host("ssh $ssh")
        Start-Process ssh $ssh
    }
}



##### MAIN #####

if ( $env:OS -eq 'Windows_NT') {
    if ($null -eq (get-command VBoxManage.exe -errorAction silentlyContinue) ) {
        $env:path = "C:\Program Files\Oracle\VirtualBox;$env:path"
    }
}


[SSH]::Config()

<#
vboxmanage list vms --long | grep -e "Name:" -e "State:"
vboxmanage list runningvms
#vboxmanage hostonlyif ipconfig $ifname --ip 192.168.56.1 --netmask 255.255.255.0
#vboxmanage dhcpserver modify --ifname $ifname --disable
#>


$id = $args[1]
if(!$id) {
    $id = "1"
}

$ids = New-Object Collections.Generic.List[String]
$ids.Add($id)
        #$vms | Foreach-Object -ThrottleLimit 3 -Parallel { $_}


$vm = [VM]::new()
$vm.Name = "xtec-$id"
$vm.SSH = "220$id"

$cmd = $args[0]
switch ($cmd) {
    ssh {
        [SSH]::connect($vm)
    }

    start {
        $vm.Start()
    }
    status {
        $vms = vboxmanage list vms
        foreach ($vm in $vms) {
            Write-Host $vm
        }
    }
    stop { 
        $vm.Stop()
    }
    update {
        [Box]::Update()
    }
    Default {
        Write-Host("Usage:     vm.ps1 [command] 
Commands:
           start id*    i.e   start, start 1, start 1 3       
           stop id*     i.e   stop, stop 2, stop 1 4    
           ssh id         
           status
")
    }
}









