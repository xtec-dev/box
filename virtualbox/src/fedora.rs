/*

https://blog.while-true-do.io/cloud-init-getting-started/

https://superuser.com/questions/928334/how-to-use-the-fedora-22-cloud-raw-raw-image-on-virtualbox

https://fedoramagazine.org/setting-up-a-vm-on-fedora-server-using-cloud-images-and-virt-install-version-3/
https://superuser.com/questions/928372/how-to-log-in-on-fedora-22-cloud-image-running-on-virtualbox


let ova_path = core::BOX_PATH.join("ova").join("box.ovf");

let output = Command::new(manage::get_cmd())
        .args([
            "storageattach",
            name,
            "--storagectl",
            "IDE",
            "--port",
            "0",
            "--device",
            "1",
            "--type",
            "dvddrive",
            "--medium",
        ])
        .arg(seed.to_path_buf())
        .output()?;
    io::stdout().write_all(&output.stdout)?;

*/
