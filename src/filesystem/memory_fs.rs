use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, UNIX_EPOCH};
use fuser::{FileAttr, Filesystem, FileType, ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory, ReplyEmpty, ReplyEntry, ReplyOpen, ReplyStatfs, ReplyWrite, Request};
use crate::filesystem::inter::node::{Data, Node};
use crate::memory;

const TTL: Duration = Duration::from_secs(1); // 1 second

pub struct MemoryFS {
    files: Arc<Mutex<HashMap<u64, Node>>>,
    next_ino: u64
}

impl MemoryFS {

    pub fn new(mut files: HashMap<u64, Node>) -> Self {
        files.insert(1, Node {
            data: Data {
                content: None,
                kind: FileType::Directory,
                size: 0
            },
            children: Some(BTreeMap::new()),
            parent: 0
        });

        let next_ino = (files.len() as u64)+1;

        Self {
            files: Arc::new(Mutex::new(files)),
            next_ino
        }
    }
}

impl Default for MemoryFS {

    fn default() -> Self {
        Self::new(HashMap::new())
    }
}

impl Filesystem for MemoryFS {

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let files = self.files.lock().unwrap();

        if let Some(ino) = files.get(&parent).as_ref().unwrap().children.as_ref().unwrap().get(name.to_str().unwrap()) {
            reply.entry(&TTL, &FileAttr {
                ino: *ino,
                size: files.get(ino).as_ref().unwrap().data.size,
                blocks: 1,
                atime: UNIX_EPOCH, // 1970-01-01 00:00:00
                mtime: UNIX_EPOCH,
                ctime: UNIX_EPOCH,
                crtime: UNIX_EPOCH,
                kind: files.get(ino).as_ref().unwrap().data.kind,
                perm: 0o777,
                nlink: 1,
                uid: 501,
                gid: 20,
                rdev: 0,
                flags: 0,
                blksize: 512
            }, 0);
            return;
        }

        reply.error(2);
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyAttr) {
        if let Some(child_node) = self.files.lock().as_ref().unwrap().get(&ino) {
            reply.attr(&TTL, &FileAttr {
                ino,
                size: child_node.data.size,
                blocks: 1,
                atime: UNIX_EPOCH, // 1970-01-01 00:00:00
                mtime: UNIX_EPOCH,
                ctime: UNIX_EPOCH,
                crtime: UNIX_EPOCH,
                kind: child_node.data.kind,
                perm: 0o777,
                nlink: 1,
                uid: 501,
                gid: 20,
                rdev: 0,
                flags: 0,
                blksize: 512
            });
        }
    }

    fn read(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, _size: u32, _flags: i32, _lock: Option<u64>, reply: ReplyData) {
        let files = self.files.lock().unwrap();

        if let Some(node) = files.get(&ino) {
            let data_len = node.data.content.as_ref().unwrap().len() as u64;

            if (offset as u64) < data_len {
                let end_offset = ((offset as usize) + (_size as usize)).min(data_len as usize);
                reply.data(&node.data.content.as_ref().unwrap()[offset as usize..end_offset]);

                return;
            }

            reply.data(&[]);
            return;
        }

        reply.error(2);
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        let files = self.files.lock().unwrap();

        if offset == 0 {
            reply.add(1, 1, FileType::Directory, ".");
            reply.add(files.get(&ino).unwrap().parent, 2, FileType::Directory, "..");
        }

        let children = files.get(&ino).unwrap().children.as_ref().unwrap().clone();

        let mut i = offset;
        for (child_name, child_ino) in children.iter().skip(i as usize) {
            if let child_node = files.get(child_ino).unwrap() {
                reply.add(*child_ino, i+2, child_node.data.kind, child_name);
                i += 1;
            }
        }

        reply.ok();
    }

    fn mkdir(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, mode: u32, umask: u32, reply: ReplyEntry) {
        let mut files = self.files.lock().unwrap();

        if !files.get(&parent).as_ref().unwrap().children.as_ref().unwrap().contains_key(name.to_str().unwrap()) {
            let ino = self.next_ino;
            self.next_ino += 1;

            files.insert(ino, Node {
                data: Data {
                    //name: name.to_str().unwrap().to_string(),
                    content: None,
                    kind: FileType::Directory,
                    size: 0
                },
                children: Some(BTreeMap::new()),
                parent: parent
            });

            files.get_mut(&parent).as_mut().unwrap().children.as_mut().unwrap().insert(name.to_str().unwrap().to_string(), ino);

            reply.entry(&TTL, &FileAttr {
                ino,
                size: 0,
                blocks: 1,
                atime: UNIX_EPOCH, // 1970-01-01 00:00:00
                mtime: UNIX_EPOCH,
                ctime: UNIX_EPOCH,
                crtime: UNIX_EPOCH,
                kind: FileType::Directory,
                perm: 0o777,
                nlink: 1,
                uid: 501,
                gid: 20,
                rdev: 0,
                flags: 0,
                blksize: 512
            }, 0);

            return;
        }

        reply.error(17);
    }

    fn create(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, mode: u32, umask: u32, flags: i32, reply: ReplyCreate) {
        let mut files = self.files.lock().unwrap();

        if files.get(&parent).as_ref().unwrap().children.as_ref().unwrap().contains_key(name.to_str().unwrap()) {
            reply.error(38);
            return;
        }

        let ino = self.next_ino;
        self.next_ino += 1;

        files.insert(ino, Node {
            data: Data {
                content: Some(Vec::new()),
                kind: FileType::RegularFile,
                size: 0
            },
            children: None,
            parent: parent
        });

        files.get_mut(&parent).as_mut().unwrap().children.as_mut().unwrap().insert(name.to_str().unwrap().to_string(), ino);

        reply.created(&TTL, &FileAttr {
            ino,
            size: 0,
            blocks: 1,
            atime: UNIX_EPOCH, // 1970-01-01 00:00:00
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: FileType::RegularFile,
            perm: 0o777,
            nlink: 1,
            uid: 501,
            gid: 20,
            rdev: 0,
            flags: 0,
            blksize: 512
        }, 0, ino, 0);
    }

    fn write(&mut self, _req: &Request<'_>, ino: u64, fh: u64, offset: i64, data: &[u8], write_flags: u32, flags: i32, lock_owner: Option<u64>, reply: ReplyWrite) {
        let mut files = self.files.lock().unwrap();

        if let Some(node) = files.get_mut(&ino) {
            let end_offset = offset as usize + data.len();
            if node.data.content.as_ref().unwrap().len() < end_offset {
                node.data.content.as_mut().unwrap().resize(end_offset, 0);
            }
            node.data.content.as_mut().unwrap()[offset as usize..end_offset].copy_from_slice(data);

            node.data.size = (offset as u64) + (data.len() as u64);
            reply.written(data.len() as u32);
            return;
        }

        reply.error(38);
    }

    fn unlink(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let mut files = self.files.lock().unwrap();

        if !files.contains_key(&parent) {
            reply.error(38);
            return;
        }

        let ino = files.get(&parent).as_ref().unwrap().children.as_ref().unwrap().get(name.to_str().unwrap()).unwrap().clone();
        files.get_mut(&parent).as_mut().unwrap().children.as_mut().unwrap().remove(name.to_str().unwrap());
        files.remove(&ino);
        reply.ok();
    }

    fn rmdir(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let mut files = self.files.lock().unwrap();

        if !files.contains_key(&parent) {
            reply.error(38);
            return;
        }

        let ino = files.get(&parent).as_ref().unwrap().children.as_ref().unwrap().get(name.to_str().unwrap()).unwrap().clone();

        let children = files.get(&ino).as_ref().unwrap().children.as_ref().unwrap().clone();
        for (child_name, child_ino) in children.iter() {
            files.remove(child_ino);
        }

        files.get_mut(&parent).as_mut().unwrap().children.as_mut().unwrap().remove(name.to_str().unwrap());
        files.remove(&ino);
        reply.ok();
    }

    fn rename(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, newparent: u64, newname: &OsStr, flags: u32, reply: ReplyEmpty) {
        let mut files = self.files.lock().unwrap();

        let ino = files.get(&parent).as_ref().unwrap().children.as_ref().unwrap().get(name.to_str().unwrap()).unwrap().clone();
        files.get_mut(&parent).as_mut().unwrap().children.as_mut().unwrap().remove(name.to_str().unwrap());

        files.get_mut(&newparent).as_mut().unwrap().children.as_mut().unwrap().insert(newname.to_str().unwrap().to_string(), ino);

        reply.ok();
    }

    fn statfs(&mut self, _req: &Request, _ino: u64, reply: ReplyStatfs) {
        let (total_ram, available_ram) = memory::get_memory_info();

        reply.statfs(
            total_ram/512, // total blocks
            available_ram/512,  // free blocks
            available_ram/512,  // available blocks
            1000000, // total inodes
            999999-self.next_ino,  // free inodes
            512,     // block size
            255,     // maximum name length
            0       // filesystem ID
        );
    }
}
