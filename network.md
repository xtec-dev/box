# network

adapter 2 
Host-only Adapter
vboxnet0

eth1 192.168.56.102/24

https://tecadmin.net/how-to-configure-static-ip-address-on-ubuntu-22-04/

/etc/netplan

network:
  ethernets:
    eth0:
      dhcp4: true
    eth1:
      addresses:
        - 192.168.56.102/24
  version: 2


sudo netplan apply 