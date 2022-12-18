#!/bash
# https://www.youtube.com/watch?v=eS_7WOnP2rs

# https://cloudinit.readthedocs.io/en/latest/topics/examples.html

genisoimage -output init.iso --input-charset utf-8 -volid cidata -joliet -rock user-data meta-data

# https://gist.github.com/leogallego/a614c61457ed22cb1d960b32de4a1b01

# ssh -i .ssh/id_ed25519_xtec  alumne@192.168.1.44
