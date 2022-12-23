use once_cell::sync::Lazy;
use std::path::PathBuf;

// https://www.virtualbox.org/sdkref/

pub static _VBOX_MANAGE: Lazy<PathBuf> = Lazy::new(|| get_cmd());

#[cfg(not(windows))]
pub fn get_cmd() -> PathBuf {
    PathBuf::from("VBoxManage")
}

#[cfg(windows)]
#[allow(dead_code)]
pub fn get_cmd() -> PathBuf {
    let mut nm = PathBuf::from("VBoxManage");
    nm.set_extension("exe");

    let exec = match std::env::var_os("VBOX_MSI_INSTALL_PATH") {
        Some(exec) => std::path::Path::new(&exec).join(&nm),
        None => nm,
    };

    exec
}
