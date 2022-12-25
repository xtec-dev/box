// https://docs.fedoraproject.org/en-US/fedora-coreos/provisioning-virtualbox/

// https://builds.coreos.fedoraproject.org/prod/streams/stable/builds/37.20221127.3.0/x86_64/fedora-coreos-37.20221127.3.0-virtualbox.x86_64.ova

//VBoxManage import --vsys 0 --vmname "$VM_NAME" fedora-coreos-37.20221127.3.0-virtualbox.x86_64.ova

// IGN_PATH="/path/to/config.ign"
// VBoxManage guestproperty set "$VM_NAME" /Ignition/Config "$(cat $IGN_PATH)"

// https://coreos.github.io/ignition/configuration-v3_2/
// https://www.airpair.com/coreos/posts/coreos-with-docker
