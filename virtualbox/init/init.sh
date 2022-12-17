#!/bash
genisoimage -output init.iso --input-charset utf-8 -volid cidata -joliet -rock user-data meta-data
