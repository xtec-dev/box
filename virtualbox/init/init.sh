#!/bin/bash

# https://www.youtube.com/watch?v=eS_7WOnP2rs

# https://cloudinit.readthedocs.io/en/latest/topics/examples.html

# https://github.com/johto/iso9660wrap

# https://gist.github.com/fardjad/a7e634d40f75dc29cff432e7372a1c93


# http://cloud-images.ubuntu.com/minimal/releases/

rm init.iso
genisoimage -output init.iso --input-charset utf-8 -volid cidata -joliet -rock user-data meta-data
rm  ~/.box/init.iso
cp init.iso ~/.box/init.iso

