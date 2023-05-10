use criterion::{Bencher, Criterion};
use lsmtree::{bytes::Bytes, KVStore, SparseMerkleTree};

use crate::{memory_store::MemoryStore, rocksdb::SmtRockSdb};

fn random_hash() -> [u8; 32] {
    monotree::utils::random_hash()
}

pub fn add_lsmtree_benches(c: &mut Criterion, sample_size: usize, tree_size: usize) {
    let mut group = c.benchmark_group("lsmtree");
    group.sample_size(sample_size);

    group.bench_function("memstore+blake3", |b| {
        test_tree(init_lsmtree_memstore_blake3(), b, tree_size)
    });

    group.bench_function("rocksdb+blake3", |b| {
        test_tree(init_lsmtree_rocksdb_blake3(), b, tree_size)
    });
}

fn fill_lsmtree<S: KVStore>(tree: &mut SparseMerkleTree<S>, nb: usize) {
    for _ in 0..nb {
        let key = random_hash();
        let leaf = random_hash();
        tree.update(key.as_slice(), Bytes::from(leaf.to_vec()))
            .unwrap();
    }
}

fn test_tree<S: KVStore>(mut tree: SparseMerkleTree<S>, b: &mut Bencher, tree_size: usize) {
    let key = random_hash();
    let leaf = random_hash();
    fill_lsmtree(&mut tree, tree_size);
    b.iter(move || {
        tree.update(key.as_slice(), Bytes::from(leaf.to_vec()))
            .unwrap();
        tree.remove(&key).unwrap();
    })
}

fn init_lsmtree_memstore_blake3() -> SparseMerkleTree<MemoryStore> {
    SparseMerkleTree::new()
}

fn init_lsmtree_rocksdb_blake3() -> SparseMerkleTree<SmtRockSdb> {
    SparseMerkleTree::new()
}
