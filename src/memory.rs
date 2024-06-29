#[cfg(target_os = "linux")]
pub fn get_memory_info() -> (u64, u64) {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open("/proc/meminfo").unwrap();
    let reader = BufReader::new(file);

    let mut total_ram: u64 = 0;
    let mut available_ram: u64 = 0;

    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with("MemTotal:") {
            total_ram = line.split_whitespace().nth(1).unwrap().parse::<u64>().unwrap() * 1024;
        } else if line.starts_with("MemAvailable:") {
            available_ram = line.split_whitespace().nth(1).unwrap().parse::<u64>().unwrap() * 1024;
        }
    }

    (total_ram, available_ram)
}

#[cfg(target_os = "windows")]
pub fn get_memory_info() -> (u64, u64) {
    use std::mem;
    use std::ptr;
    use winapi::um::sysinfoapi::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    unsafe {
        let mut mem_info: MEMORYSTATUSEX = mem::zeroed();
        mem_info.dwLength = mem::size_of::<MEMORYSTATUSEX>() as u32;
        if GlobalMemoryStatusEx(&mut mem_info as *mut MEMORYSTATUSEX as *mut std::ffi::c_void) != 0 {
            let total_ram = mem_info.ullTotalPhys;
            let available_ram = mem_info.ullAvailPhys;

            (total_ram, available_ram)
        } else {
            (0, 0)
        }
    }
}

#[cfg(target_os = "macos")]
pub fn get_memory_info() -> (u64, u64) {
    use std::process::Command;
    use std::str;

    let total_ram_output = Command::new("sysctl")
        .arg("hw.memsize")
        .output()
        .expect("Failed to execute sysctl");
    let total_ram_str = str::from_utf8(&total_ram_output.stdout).expect("Invalid UTF-8");
    let total_ram: u64 = total_ram_str
        .trim()
        .split(':')
        .nth(1)
        .unwrap()
        .trim()
        .parse()
        .unwrap();

    let vm_stat_output = Command::new("vm_stat")
        .output()
        .expect("Failed to execute vm_stat");
    let vm_stat_str = str::from_utf8(&vm_stat_output.stdout).expect("Invalid UTF-8");

    let page_size = 4096; // Default page size in bytes on macOS
    let mut free_pages: u64 = 0;

    for line in vm_stat_str.lines() {
        if line.starts_with("Pages free:") {
            free_pages = line
                .split_whitespace()
                .last()
                .unwrap()
                .trim_end_matches('.')
                .parse()
                .unwrap();
        }
    }

    let available_ram = free_pages * page_size;

    (total_ram, available_ram)
}
