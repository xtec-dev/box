#!/bin/bash

mkdir -p tmp/ova
cd tmp
#wget https://download.fedoraproject.org/pub/fedora/linux/releases/37/Cloud/x86_64/images/Fedora-Cloud-Base-Vagrant-37-1.7.x86_64.vagrant-virtualbox.box -o fedora-37.box
tar xf fedora-37.box
rm metadata.json
rm Vagrantfile
tar cf fedora-37.ova box.ovf Fedora-Cloud-Base-Vagrant-37-1.7.x86_64.vmdk



#tar cf fedora-37.ova 

