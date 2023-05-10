use lsmtree::{bytes::Bytes, BadProof};
use monotree::Database;
use rand::Rng;
use sparse_merkle_tree::{
    merge::MergeValue,
    traits::{StoreReadOps, StoreWriteOps},
    BranchKey, BranchNode, H256,
};

use crate::blake3::Blake3SmtHasher;

pub struct SmtRockSdb {
    db: rocksdb::DB,
    branch_db: rocksdb::DB,
}

impl Default for SmtRockSdb {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        let rand_nb: u64 = rng.gen();
        SmtRockSdb::new(format!(".bench_db/rocksdb_default_{}", rand_nb).as_str())
    }
}

// MONOTREE

impl Database for SmtRockSdb {
    fn new(dbpath: &str) -> Self {
        SmtRockSdb {
            db: rocksdb::DB::open_default(dbpath).unwrap(),
            branch_db: rocksdb::DB::open_default(format!("{}_-branch", dbpath)).unwrap(),
        }
    }

    fn get(&mut self, key: &[u8]) -> monotree::Result<Option<Vec<u8>>> {
        Ok(self.db.get(key).unwrap())
    }

    fn put(&mut self, key: &[u8], value: Vec<u8>) -> monotree::Result<()> {
        self.db.put(key, value).unwrap();
        Ok(())
    }

    fn delete(&mut self, key: &[u8]) -> monotree::Result<()> {
        self.db.delete(key).unwrap();
        Ok(())
    }

    fn init_batch(&mut self) -> monotree::Result<()> {
        Ok(())
    }

    fn finish_batch(&mut self) -> monotree::Result<()> {
        Ok(())
    }
}

// SPARSE MERKLE TREE

fn serialize_smt_branchnode_arm(arm: MergeValue, vec: &mut Vec<u8>) {
    match arm {
        MergeValue::Value(hash) => {
            vec.push(1);
            vec.extend(hash.as_slice().to_vec());
        }
        MergeValue::MergeWithZero {
            base_node,
            zero_bits,
            zero_count,
        } => {
            vec.push(2);
            vec.extend(base_node.as_slice().to_vec());
            vec.extend(zero_bits.as_slice().to_vec());
            vec.push(zero_count);
        }
    }
}

fn get_32_bytes<'a, I: Iterator<Item = &'a u8>>(bytes: &mut I) -> [u8; 32] {
    let data = bytes.take(32).cloned().collect::<Vec<u8>>();
    debug_assert_eq!(data.len(), 32);
    data.try_into().unwrap()
}

fn merge_value_from_bytes<'a, I: Iterator<Item = &'a u8>>(bytes: &mut I) -> Result<MergeValue, ()> {
    match *bytes.next().unwrap() {
        1 => Ok(MergeValue::Value(H256::from(get_32_bytes(bytes)))),
        2 => Ok(MergeValue::MergeWithZero {
            base_node: H256::from(get_32_bytes(bytes)),
            zero_bits: H256::from(get_32_bytes(bytes)),
            zero_count: *bytes.next().unwrap(),
        }),
        _ => Err(()),
    }
}

fn deserialize_smt_branchnode_arm(data: Vec<u8>) -> Option<(MergeValue, MergeValue)> {
    let mut bytes = data.iter();
    let larm_res = merge_value_from_bytes(&mut bytes);
    if larm_res.is_err() {
        return None;
    }
    let larm = larm_res.unwrap();
    let rarm_res = merge_value_from_bytes(&mut bytes);
    if rarm_res.is_err() {
        return None;
    }
    let rarm = rarm_res.unwrap();
    Some((larm, rarm))
}

impl StoreWriteOps<H256> for SmtRockSdb {
    fn insert_branch(
        &mut self,
        node_key: BranchKey,
        branch: BranchNode,
    ) -> Result<(), sparse_merkle_tree::error::Error> {
        let mut buffer = vec![];
        serialize_smt_branchnode_arm(branch.left, &mut buffer);
        serialize_smt_branchnode_arm(branch.right, &mut buffer);
        self.branch_db
            .put(node_key.node_key.as_slice(), buffer)
            .unwrap();
        Ok(())
    }

    fn insert_leaf(
        &mut self,
        leaf_key: H256,
        leaf: H256,
    ) -> Result<(), sparse_merkle_tree::error::Error> {
        self.db.put(leaf_key.as_slice(), leaf.as_slice()).unwrap();
        Ok(())
    }

    fn remove_branch(
        &mut self,
        node_key: &BranchKey,
    ) -> Result<(), sparse_merkle_tree::error::Error> {
        self.branch_db.delete(node_key.node_key.as_slice()).unwrap();
        Ok(())
    }

    fn remove_leaf(&mut self, leaf_key: &H256) -> Result<(), sparse_merkle_tree::error::Error> {
        self.db.delete(leaf_key.as_slice()).unwrap();
        Ok(())
    }
}

impl StoreReadOps<H256> for SmtRockSdb {
    fn get_branch(
        &self,
        branch_key: &BranchKey,
    ) -> Result<Option<BranchNode>, sparse_merkle_tree::error::Error> {
        if let Some(data) = self.branch_db.get(branch_key.node_key.as_slice()).unwrap() {
            if let Some((larm, rarm)) = deserialize_smt_branchnode_arm(data) {
                let branchnode = BranchNode {
                    left: larm,
                    right: rarm,
                };
                Ok(Some(branchnode))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn get_leaf(&self, leaf_key: &H256) -> Result<Option<H256>, sparse_merkle_tree::error::Error> {
        let data = self.db.get(leaf_key.as_slice()).unwrap();
        Ok(data.map(|d| {
            let array: [u8; 32] = d.try_into().unwrap();
            array.into()
        }))
    }
}

// LSMTREE

impl lsmtree::KVStore for SmtRockSdb {
    type Hasher = Blake3SmtHasher;
    type Error = BadProof;

    fn get(&self, key: &[u8]) -> Result<Option<lsmtree::bytes::Bytes>, Self::Error> {
        Ok(self.db.get(key).unwrap().map(Bytes::from))
    }

    fn set(
        &mut self,
        key: lsmtree::bytes::Bytes,
        value: lsmtree::bytes::Bytes,
    ) -> Result<(), Self::Error> {
        self.db.put(key.to_vec().as_slice(), &value).unwrap();
        Ok(())
    }

    fn remove(&mut self, key: &[u8]) -> Result<lsmtree::bytes::Bytes, Self::Error> {
        let content = Bytes::from(self.get(key).unwrap().unwrap());
        self.db.delete(key).unwrap();
        Ok(content)
    }

    fn contains(&self, key: &[u8]) -> Result<bool, Self::Error> {
        Ok(self.db.key_may_exist(key))
    }
}
