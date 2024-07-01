use std::env;
use std::io::BufRead;
use std::path::Path;
use std::process::exit;

pub mod filesystem;
pub mod daemon;
mod memory;

use filesystem::memory_fs::MemoryFS;
use fuser::{MountOption};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <FOLDER_PATH>", args[0]);
        exit(1);
    }

    let mountpoint = &args[1];

    let path = Path::new(mountpoint);
    if !path.exists() {
        println!("The path does not exist.");
    }

    if let Err(err) = daemon::daemonize() {
        eprintln!("Daemonization failed: {}", err);
        exit(1);
    }

    let options = [
        MountOption::RW,
        MountOption::FSName("MemoryFS".to_string()),
        MountOption::Async
        //MountOption::AutoUnmount
    ];

    fuser::mount2(MemoryFS::default(), mountpoint, &options).unwrap();
}
