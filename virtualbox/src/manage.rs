use std::path::{Path, PathBuf};
use once_cell::sync::Lazy;

pub static VBoxManage:Lazy<PathBuf> = Lazy::new(|| {get_cmd()});


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
    Some(exec) => Path::new(&exec).join(&nm),
    None => nm
  };

  exec
}
