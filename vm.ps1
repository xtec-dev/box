#
# Add Virtual Box bin-path to PATH environment variable if necessary:
#
if ( $IsWindows &&  (get-command VBoxManage.exe -errorAction silentlyContinue) -eq $null) {
    $env:path="C:\Program Files\Oracle\VirtualBox;$env:path"
}

Write-Host(VBoxManage startvm xtec-1 --type headless)



# https://www.how2shout.com/how-to/vboxmanage-command-not-found-in-windows-cmd-or-powershell.html
