use std::path::PathBuf;
use structopt::StructOpt;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(
    name = "mkisofs-rs",
    about = "create an hybrid ISO-9660 filesystem-image with Rock Ridge attributes."
)]
pub struct Opt {
    #[structopt(long, short = "o", help = "Set output file name")]
    pub output: String,

    #[structopt(parse(from_os_str))]
    pub input_files: Vec<PathBuf>,
}
