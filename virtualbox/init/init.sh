#!/bash
# https://www.youtube.com/watch?v=eS_7WOnP2rs

# https://cloudinit.readthedocs.io/en/latest/topics/examples.html

# https://github.com/frederickding/Cloud-Init-ISO/blob/master/build.sh

#genisoimage -output init.iso --input-charset utf-8 -volid cidata -joliet -rock user-data meta-data

rm  ~/.box/init.iso
cp init.iso ~/.box/init.iso

# https://gist.github.com/leogallego/a614c61457ed22cb1d960b32de4a1b01

