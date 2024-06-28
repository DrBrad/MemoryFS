Memory FileSystem
-----
MemoryFS is a simple and easy to use memory based filesystem, this means any files within this filesystem are temporary and will only last as long as the drive is mounted and the computer is running. This is great if you need to hold temporary files or make modifications to files without effecting your hard drives life span. Another great usage for this would be if you need to access files allot faster than with your hard drive.

Requirements
=====
You will need to have Rust installed to build.
You will need to have Fuse installed This is available for (Windows, MacOS, and many Linux Distros).

This is the command to install and build within Ubuntu
```
sudo apt-get install fuse libfuse-dev
```

Installation
=====
To install this go to the directory and run:
```
cargo build --release
```

If your on linux and you want to make this a command type:
```
cp MemoryFS/target/debug/MemoryFS /usr/local/bin/memoryfs
```

Usage
=====
The usage is simple
```
memoryfs PATH/TO/MOUNT
```
