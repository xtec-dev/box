# network

adapter 2 
Host-only Adapter
vboxnet0

eth1 192.168.56.102/24

https://tecadmin.net/how-to-configure-static-ip-address-on-ubuntu-22-04/

/etc/netplan

echo "
network:
  ethernets:
    eth1:
      addresses:
        - 192.168.56.102/24
  version: 2
" | sudo tee /etc/netplan/00-box.yaml

sudo netplan apply 