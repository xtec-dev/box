

#VBoxManage showvminfo $vmName --machinereadable 

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
        [string]$FileDestination)

    # set protocol to tls version 1.2
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

    # Download the Virus Warning into _tmp.txt
    Invoke-WebRequest -Uri "https://drive.google.com/uc?export=download&id=$GoogleFileId" -OutFile "_tmp.txt" -SessionVariable googleDriveSession

    # Get confirmation code from _tmp.txt
    $searchString = Select-String -Path "_tmp.txt" -Pattern "confirm="
    $searchString -match "confirm=(?<content>.*)&amp;id="
    $confirmCode = $matches['content']

    # Delete _tmp.txt
    Remove-Item "_tmp.txt"

    # Download the real file
    Invoke-WebRequest -Uri "https://drive.google.com/uc?export=download&confirm=${confirmCode}&id=$GoogleFileId" -OutFile $FileDestination -WebSession $googleDriveSession
}

##### MAIN #####

if ( $IsWindows) {
    if ($null -eq (get-command VBoxManage.exe -errorAction silentlyContinue) ) {
        $env:path = "C:\Program Files\Oracle\VirtualBox;$env:path"
    }
}



<#
$ova = "xtec.ova"

if (-not(Test-Path $ova -PathType Leaf)) {
    Write-Host("Downloading xtec.ova from Google Drive")
    DriveDownload -GoogleFileId "1UxNLsSvv7eo-M6MmAgadn7m14wEvrMmZ" -FileDestionatoin "xtec.ova"
}
#>

# VBoxManage clonevm $vmName --name=$vmNameClone --register

# vboxmanage list vms
$vms = New-Object Collections.Generic.List[VM]
foreach ($i in 1..1) {
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

    $vm.Stop()
    
    vboxmanage modifyvm $vm.Name --nic1 nat
    vboxmanage modifyvm $vm.Name --natpf1 delete ssh
    vboxmanage modifyvm $vm.Name --natpf1 "ssh,tcp,127.0.0.1,$($vm.SSH),,22"
    
    vboxmanage modifyvm $vm.Name --nic2 hostonly --hostonlyadapter2 $ifname
    $vm.Start()
}

#$vms | Foreach-Object -ThrottleLimit 3 -Parallel { $_}








