use std::collections::{BTreeMap};
use fuser::{FileType};

pub struct Node {
    pub data: Data,
    pub children: Option<BTreeMap<String, u64>>,
    pub parent: u64
}

pub struct Data {
    //pub name: String,
    pub content: Option<Vec<u8>>,
    pub kind: FileType,
    pub size: u64
}
