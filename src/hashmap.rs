use std::collections::BTreeMap;

pub struct MemoryStore {
    db: BTreeMap<[u8; 32], Vec<u8>>,
}

impl Database for MemoryStore {
    
}
