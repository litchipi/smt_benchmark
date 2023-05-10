use criterion::{Bencher, Criterion};
use sparse_merkle_tree::traits::{Hasher, StoreReadOps, StoreWriteOps};
use sparse_merkle_tree::{SparseMerkleTree, H256};

use crate::blake3::Blake3SmtHasher;
use crate::memory_store::MemoryStore;
use crate::rocksdb::SmtRockSdb;

fn random_hash() -> H256 {
    monotree::utils::random_hash().into()
}

pub fn add_sparse_merkle_tree_benches(c: &mut Criterion, sample_size: usize, tree_size: usize) {
    let mut group = c.benchmark_group("sparse-merkle-tree");
    group.sample_size(sample_size);

    group.bench_function("memstore+blake3", |b| {
        test_tree(init_sparse_merkle_tree_blake3_memorystore(), b, tree_size)
    });

    group.bench_function("rocksdb+blake3", |b| {
        test_tree(init_sparse_merkle_tree_blake3_rocksdb(), b, tree_size)
    });
}

fn fill_smt<H, S>(tree: &mut SparseMerkleTree<H, H256, S>, nb: usize)
where
    H: Hasher + Default,
    S: StoreReadOps<H256> + StoreWriteOps<H256>,
{
    for _ in 0..nb {
        let key = random_hash();
        let val = random_hash();
        tree.update(key, val).unwrap();
    }
}

fn test_tree<H, S>(mut tree: SparseMerkleTree<H, H256, S>, b: &mut Bencher, tree_size: usize)
where
    H: Hasher + Default,
    S: StoreReadOps<H256> + StoreWriteOps<H256>,
{
    fill_smt(&mut tree, tree_size);
    let key = random_hash();
    let leaf = random_hash();
    b.iter(move || {
        tree.update(key, leaf).unwrap();
        let _ = tree.get(&key).unwrap();
        tree.store_mut().remove_leaf(&key).unwrap();
    })
}

fn init_sparse_merkle_tree_blake3_memorystore(
) -> SparseMerkleTree<Blake3SmtHasher, H256, MemoryStore> {
    SparseMerkleTree::default()
}

fn init_sparse_merkle_tree_blake3_rocksdb() -> SparseMerkleTree<Blake3SmtHasher, H256, SmtRockSdb> {
    SparseMerkleTree::default()
}
