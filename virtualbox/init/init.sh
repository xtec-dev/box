#!/bin/bash

# https://www.youtube.com/watch?v=eS_7WOnP2rs

# https://cloudinit.readthedocs.io/en/latest/topics/examples.html

# https://github.com/frederickding/Cloud-Init-ISO/blob/master/build.sh

# https://github.com/johto/iso9660wrap

rm init.iso
genisoimage -output init.iso --input-charset utf-8 -volid cidata -joliet -rock user-data meta-data
rm  ~/.box/init.iso
cp init.iso ~/.box/init.iso

