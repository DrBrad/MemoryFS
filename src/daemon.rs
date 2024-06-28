use std::{env, io};
use std::fs::OpenOptions;
use std::os::fd::AsRawFd;
use std::process::exit;
use libc::{fork, setsid, dup2};

pub fn daemonize() -> io::Result<()> {
    if let pid = unsafe {
        fork()
    } {
        if pid > 0 {
            exit(0);
        }
    } else {
        eprintln!("Fork failed");
        exit(1);
    }

    if unsafe {
        setsid()
    } < 0 {
        eprintln!("Failed to create a new session");
        exit(1);
    }

    env::set_current_dir("/")?;

    let dev_null = OpenOptions::new().read(true).write(true).open("/dev/null")?;
    for fd in 0..3 {
        if unsafe {
            dup2(dev_null.as_raw_fd(), fd)
        } < 0 {
            eprintln!("Failed to redirect file descriptor {}", fd);
            exit(1);
        }
    }

    Ok(())
}
