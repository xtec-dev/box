#!/bin/bash

# wget https://builds.coreos.fedoraproject.org/prod/streams/stable/builds/37.20221127.3.0/x86_64/fedora-coreos-37.20221127.3.0-virtualbox.x86_64.ova coreos-37.ova

# podman run --pull=always --rm -i quay.io/coreos/ignition-validate:release - < config.ign

# mkpasswd

VBoxManage import --vsys 0 --vmname box-5 coreos-37.ova
VBoxManage guestproperty set box-5 /Ignition/Config "$(cat config.ign)"
VBoxManage modifyvm box-5 --natpf1 "guestssh,tcp,,2205,,22"



