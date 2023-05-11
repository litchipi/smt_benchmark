use criterion::{Bencher, Criterion};
use monotree::utils::random_hash;
use monotree::{Database, Hash, Hasher, Monotree};

use crate::blake3::Blake3SmtHasher;
use crate::memory_store::MemoryStore;
use crate::rocksdb::SmtRockSdb;

// Blake3 > Sha256 > other hash functions

pub fn add_monotree_benches(c: &mut Criterion, sample_size: usize, tree_size: usize) {
    let mut group = c.benchmark_group("monotree");
    group.sample_size(sample_size);

    group.bench_function("memstore+blake3", |b| {
        test_tree(init_monotree_memstore_blake3(), b, tree_size)
    });

    group.bench_function("memstore+blake3/read", |b| {
        test_tree_read_only(init_monotree_memstore_blake3(), b, tree_size)
    });

    group.bench_function("memstore+blake3/write", |b| {
        test_tree_write_only(init_monotree_memstore_blake3(), b, tree_size)
    });

    group.bench_function("rocksdb+blake3", |b| {
        test_tree(init_monotree_rocksdb_blake3(), b, tree_size)
    });

    group.bench_function("rocksdb+blake3/read", |b| {
        test_tree_read_only(init_monotree_rocksdb_blake3(), b, tree_size)
    });

    group.bench_function("rocksdb+blake3/write", |b| {
        test_tree_write_only(init_monotree_rocksdb_blake3(), b, tree_size)
    });
}

fn fill_monotree<D: Database, H: Hasher>(tree: &mut Monotree<D, H>, nb: usize) -> Option<Hash> {
    let mut root = None;
    for _ in 0..nb {
        let key = random_hash();
        let leaf = random_hash();
        root = tree.insert(root.as_ref(), &key, &leaf).unwrap();
        assert_ne!(root, None);
    }
    root
}

fn test_tree<D: Database, H: Hasher>(mut tree: Monotree<D, H>, b: &mut Bencher, tree_size: usize) {
    let root = fill_monotree(&mut tree, tree_size);
    let key = random_hash();
    let leaf = random_hash();
    b.iter(move || {
        let new_root = tree.insert(root.as_ref(), &key, &leaf).unwrap();
        let _ = tree.get(new_root.as_ref(), &key).unwrap();
        tree.remove(new_root.as_ref(), &key).unwrap();
    })
}

fn test_tree_read_only<D: Database, H: Hasher>(mut tree: Monotree<D, H>, b: &mut Bencher, tree_size: usize) {
    let root = fill_monotree(&mut tree, tree_size);
    let key = random_hash();
    let leaf = random_hash();
    let new_root = tree.insert(root.as_ref(), &key, &leaf).unwrap();
    b.iter(move || {
        let _ = tree.get(new_root.as_ref(), &key).unwrap();
    })
}

fn test_tree_write_only<D: Database, H: Hasher>(mut tree: Monotree<D, H>, b: &mut Bencher, tree_size: usize) {
    let root = fill_monotree(&mut tree, tree_size);
    let key = random_hash();
    let leaf = random_hash();
    b.iter(move || {
        let new_root = tree.insert(root.as_ref(), &key, &leaf).unwrap();
        tree.remove(new_root.as_ref(), &key).unwrap();
    })
}

fn init_monotree_memstore_blake3() -> Monotree<MemoryStore, Blake3SmtHasher> {
    Monotree::new("./.bench_db/monotree_hashmap_blake3")
}

fn init_monotree_rocksdb_blake3() -> Monotree<SmtRockSdb, Blake3SmtHasher> {
    Monotree::new("./.bench_db/monotree_rocksdb_blake3")
}

// TODO    Add SHA2 flavor also
