use std::env;
use std::path::Path;
pub mod filesystem;

use filesystem::memory_fs::MemoryFS;
use fuser::{MountOption};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <FOLDER_PATH>", args[0]);
        std::process::exit(1);
    }

    let mountpoint = &args[1];

    let path = Path::new(mountpoint);
    if !path.exists() {
        println!("The path does not exist.");
    }

    let mut options = [
        MountOption::RW,
        MountOption::FSName("KFS".to_string()),
        MountOption::Async
        //MountOption::AutoUnmount
    ];
    fuser::mount2(MemoryFS::default(), mountpoint, &options).unwrap();
}
