param($cmd = "help")

Write-Verbose "Creating the VirtualBox COM object"
$global:vbox = New-Object -ComObject "VirtualBox.VirtualBox"

# TODO config memory and CPU
# TODO Change default pk with user pk

# Format: Shit + Alt + F


class Box {

    hidden static [String] $home = "$global:HOME/.xtec"
    hidden static [String] $ova = "$([Box]::home)/xtec.ova"

    static [String] Adapter() {

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

        return $adapter
    }


    static [void] Import( [string] $Name) {
    
        if (-not(Test-Path ([Box]::ova) -PathType Leaf)) {
            [Box]::Update()
        }

        Write-Host("box: importing virtual machine $Name")
        vboxmanage import ([Box]::ova) --vsys 0 --vmname $Name --basefolder ([Box]::home)
    }

    static [void] List() {
        $vms = $global:vbox.Machines
        foreach ($vm in $vms) {
            if ($vm.Name -like 'xtec-*') {
                $state = $vm.State
                $state = switch ($state) {
                    1 { "stopped" }
                    2 { "saved" }
                    5 { "running" }
                    9 { "starting" }
                    10 { "stopping" }
                    11 { "restoring" }
                    Default { $state }
                }

                Write-Host("$($vm.Name): $state")
            }
        }
            
    }

    static [void] Update() {
        if (!(Test-Path -PathType Container ([Box]::home))) {
            New-Item -ItemType Directory -Path ([Box]::home) 
        }
        
        Write-Host("box: downloading xtec.ova")
        [Box]::Download("1UxNLsSvv7eo-M6MmAgadn7m14wEvrMmZ", ([Box]::ova))
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
    [String]$Id
    [String]$Name
    [String]$SSH

    [void] Delete() {
        $this.Stop()
        Write-Host(vboxmanage unregister $this.name --delete)
        #rmdir -recurse $vmPath
        #rmdir -recurse $sharedFolder
    }

    [void] Start() {

        $vm = $null
        try {
            $vm = $global:vbox.FindMachine($($this.Name))
        }
        catch {
            Write-Host("$($this.Name): machine it's not registered")
            [Box]::Import($this.Name)
            $vm = $global:vbox.FindMachine($($this.Name))
        }
        

        # Stopped                
        if ($vm.State -eq 1) {

            vboxmanage modifyvm $this.Name --memory 4096
            vboxmanage modifyvm $this.Name --cpus 2
            vboxmanage modifyvm $this.Name --nic1 nat
            vboxmanage modifyvm $this.Name --natpf1 delete ssh
            vboxmanage modifyvm $this.Name --natpf1 "ssh,tcp,127.0.0.1,$($this.SSH),,22"

            $adapter = [Box]::Adapter()
            vboxmanage modifyvm $this.Name --nic2 hostonly --hostonlyadapter2 $adapter
        }


        # Running
        if (-not($vm.State -eq 5)) {
            Write-Host("$($this.Name): starting machine ...")
            Write-Host(vboxmanage startvm $this.Name --type headless)
        }    
                
        Write-Host("$($this.Name): waiting ssh ready ...")
        [SSH]::Execute($this, "sudo hostnamectl set-hostname $($this.Name); echo '
        network:
          ethernets:
            eth1:
              addresses:
                - 192.168.56.10$($this.id)/24
          version: 2
        ' | sudo tee /etc/netplan/10-box.yaml; sudo netplan apply")
    }

    [void] Stop() {

        try {
            $vm = $global:vbox.FindMachine($($this.Name))
            if (-not($vm.State -eq 1)) {
                Write-Host("$($this.Name): waiting to poweroff machine ...")
                vboxmanage controlvm $this.Name acpipowerbutton
            }
        }
        catch {
            Write-Host("$($this.Name): machine it's not registered")
        }

        <#
        $info = VBoxManage showvminfo --machinereadable $this.Name
        while (-not($info -like 'VMState="poweroff"')) {
            vboxmanage controlvm $this.Name acpipowerbutton
            Write-Host("$($this.Name): waiting to poweroff machine ...")
            Start-Sleep -Seconds 2
            $info = VBoxManage showvminfo --machinereadable $this.Name
        }
        #>
    }
}



class SSH {

    hidden static [String] $key = "$global:HOME/.ssh/id_ed25519_xtec" 

    static [void] Config() {
    
        $file = [SSH]::key

        if (-not(Test-Path $file -PathType Leaf)) {
            New-Item -Path $file -ItemType File -Force

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
    }

    static [void] Execute([VM] $vm, [String] $cmd) {

        ssh -p $($vm.SSH) -i $([SSH]::key) -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no alumne@127.0.0.1 "$cmd"
    }
}



##### MAIN #####


if ($null -eq (get-command VBoxManage.exe -errorAction silentlyContinue) ) {
    $env:path = "C:\Program Files\Oracle\VirtualBox;$env:path"
}


[SSH]::Config()


$vms = New-Object Collections.Generic.List[VM]
foreach ($id in $args) {
    $vm = [VM]::new()
    $vm.Id = $id
    $vm.Name = "xtec-$id"
    $vm.SSH = "220$id"
    $vms.Add($vm)
}

if ($vms.count -eq 0) {
    $id = 1
    $vm = [VM]::new()
    $vm.Id = $id
    $vm.Name = "xtec-$id"
    $vm.SSH = "220$id"
    $vms.Add($vm)
}

switch ($cmd) {
    ssh {
        [SSH]::Connect($vm)
    }

    start {
        foreach ($vm in $vms) {
            $vm.Start()
        }
    }
    pstart {
        #$vms | Foreach-Object -ThrottleLimit 3 -Parallel { $_}
        Workflow StartParallel {
            Foreach -parallel($vm in $vms) {
                InlineScript {
                    $vm.Start()
                }
            }
        }
    }
    list {
        [Box]::List()
    }
    stop { 
        foreach ($vm in $vms) {
            $vm.Stop()
        }
    }
    pstop {
        Workflow StopParallel {
            Foreach -parallel($vm in $vms) {
                InlineScript {
                    $vm.Stop()
                }
            }
        }
    }
    update {
        [Box]::Update()
    }
    install {
        Copy-Item box.ps1 "C:\Users\david\AppData\Local\Microsoft\Windows"
    }

    Default {
        Write-Host("Usage:     box.ps1 [command] 
Commands:
           start id*  default 1       
           stop id*   default 1
           ssh id   1       
           list
")
    }
}









