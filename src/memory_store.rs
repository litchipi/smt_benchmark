use std::collections::BTreeMap;

use lsmtree::{bytes::Bytes, BadProof};
use sparse_merkle_tree::{
    traits::{StoreReadOps, StoreWriteOps},
    H256,
};

use crate::blake3::Blake3SmtHasher;

#[derive(Default)]
pub struct MemoryStore {
    db: BTreeMap<[u8; 32], Vec<u8>>,
    branch_db: BTreeMap<sparse_merkle_tree::BranchKey, sparse_merkle_tree::BranchNode>,
}

// MONOTREE

impl monotree::Database for MemoryStore {
    fn new(_: &str) -> Self {
        MemoryStore {
            db: BTreeMap::new(),
            branch_db: BTreeMap::new(),
        }
    }

    fn get(&mut self, key: &[u8]) -> monotree::Result<Option<Vec<u8>>> {
        Ok(self.db.get(key).cloned())
    }

    fn put(&mut self, key: &[u8], value: Vec<u8>) -> monotree::Result<()> {
        let key = key.try_into().unwrap();
        self.db.insert(key, value);
        Ok(())
    }

    fn delete(&mut self, key: &[u8]) -> monotree::Result<()> {
        self.db.remove(key);
        Ok(())
    }

    fn init_batch(&mut self) -> monotree::Result<()> {
        Ok(())
    }

    fn finish_batch(&mut self) -> monotree::Result<()> {
        Ok(())
    }
}

// Sparse Merkle Tree

impl StoreReadOps<H256> for MemoryStore {
    fn get_branch(
        &self,
        branch_key: &sparse_merkle_tree::BranchKey,
    ) -> Result<Option<sparse_merkle_tree::BranchNode>, sparse_merkle_tree::error::Error> {
        Ok(self.branch_db.get(branch_key).cloned())
    }

    fn get_leaf(
        &self,
        leaf_key: &sparse_merkle_tree::H256,
    ) -> Result<Option<H256>, sparse_merkle_tree::error::Error> {
        Ok(self.db.get(leaf_key.as_slice()).map(|v| {
            let buff: [u8; 32] = v
                .iter()
                .take(32)
                .cloned()
                .collect::<Vec<u8>>()
                .try_into()
                .unwrap();
            H256::from(buff)
        }))
    }
}

impl StoreWriteOps<H256> for MemoryStore {
    fn insert_branch(
        &mut self,
        node_key: sparse_merkle_tree::BranchKey,
        branch: sparse_merkle_tree::BranchNode,
    ) -> Result<(), sparse_merkle_tree::error::Error> {
        self.branch_db.insert(node_key, branch);
        Ok(())
    }

    fn insert_leaf(
        &mut self,
        leaf_key: sparse_merkle_tree::H256,
        leaf: H256,
    ) -> Result<(), sparse_merkle_tree::error::Error> {
        let key: [u8; 32] = leaf_key.as_slice().try_into().unwrap();
        self.db.insert(key, leaf.as_slice().to_vec());
        Ok(())
    }

    fn remove_branch(
        &mut self,
        node_key: &sparse_merkle_tree::BranchKey,
    ) -> Result<(), sparse_merkle_tree::error::Error> {
        self.branch_db.remove(node_key);
        Ok(())
    }

    fn remove_leaf(
        &mut self,
        leaf_key: &sparse_merkle_tree::H256,
    ) -> Result<(), sparse_merkle_tree::error::Error> {
        self.db.remove(leaf_key.as_slice());
        Ok(())
    }
}

// LSMTREE

impl lsmtree::KVStore for MemoryStore {
    type Hasher = Blake3SmtHasher;
    type Error = BadProof;

    fn get(&self, key: &[u8]) -> Result<Option<lsmtree::bytes::Bytes>, Self::Error> {
        let key: [u8; 32] = key.try_into().unwrap();
        Ok(self.db.get(&key).map(|b| Bytes::from(b.clone())))
    }

    fn set(
        &mut self,
        key: lsmtree::bytes::Bytes,
        value: lsmtree::bytes::Bytes,
    ) -> Result<(), Self::Error> {
        let key: [u8; 32] = key.to_vec().try_into().unwrap();
        self.db.insert(key, value.to_vec());
        Ok(())
    }

    fn remove(&mut self, key: &[u8]) -> Result<lsmtree::bytes::Bytes, Self::Error> {
        let key: [u8; 32] = key.try_into().unwrap();
        Ok(Bytes::from(self.db.remove(&key).unwrap()))
    }

    fn contains(&self, key: &[u8]) -> Result<bool, Self::Error> {
        let key: [u8; 32] = key.try_into().unwrap();
        Ok(self.db.contains_key(&key))
    }
}
