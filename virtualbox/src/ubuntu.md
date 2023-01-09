# Ubuntu

``` after first boot
sudo touch /etc/cloud/cloud-init.disabled
```

https://cloudinit.readthedocs.io/en/latest/topics/analyze.html

cloud-init analyze boot

$ systemd-analyze time
$ systemd-analyze critical-chain
$ systemd-analyze blame

systemctl disable snapd

systemctl show -p WantedBy network-online.target



sudo systemctl disable snapd.service
sudo systemctl disable snapd.socket
sudo systemctl disable snapd.seeded.service


snapd https://discuss.linuxcontainers.org/t/cloud-init-is-blocked-by-snapd-in-the-new-ubuntu-images/4571 
https://ubuntuforums.org/showthread.php?t=2475667

https://askubuntu.com/questions/1280508/detect-and-fix-slow-startup-on-ubuntu-20-04

- snapd.service @20.189s +22.924s
─systemd-networkd-wait-online.service @4.761s +13.805s

https://cloudinit.readthedocs.io/en/latest/topics/faq.html


https://ubuntuforums.org/showthread.php?t=2475667

network:
  ethernets:
    ens160:
      optional: true
      dhcp4: false
      dhcp6: false
      addresses:
      - 192.168.1.2/24
      routes: 
      - to: default
        via: 192.168.1.1
      nameservers:
        addresses:
        - 1.1.1.1
        - 8.8.8.8
        search: []
  version: 2
