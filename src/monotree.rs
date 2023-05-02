use criterion::{Bencher, Criterion};
use monotree::database::rocksdb::RocksDB;
use monotree::database::sled::Sled;
use monotree::database::MemoryDB;
use monotree::hasher::Blake3;
use monotree::utils::random_hash;
use monotree::{Database, Hash, Hasher, Monotree};

// Blake3 > Sha256 > other hash functions

pub fn add_monotree_benches(c: &mut Criterion, sample_size: usize) {
    let mut group = c.benchmark_group("monotree");
    group.sample_size(sample_size);

    group.bench_function("hashmap+blake3", |b| {
        test_tree(init_monotree_hashmap_blake3(), b)
    });

    group.bench_function("rocksdb+blake3", |b| {
        test_tree(init_monotree_rocksdb_blake3(), b)
    });

    group.bench_function("sled+blake3", |b| test_tree(init_monotree_sled_blake3(), b));
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

fn test_tree<D: Database, H: Hasher>(mut tree: Monotree<D, H>, b: &mut Bencher) {
    let root = fill_monotree(&mut tree, 1000);
    let key = random_hash();
    let leaf = random_hash();
    b.iter(move || {
        let new_root = tree.insert(root.as_ref(), &key, &leaf).unwrap();
        tree.remove(new_root.as_ref(), &key).unwrap();
    })
}

fn init_monotree_hashmap_blake3() -> Monotree<MemoryDB, Blake3> {
    Monotree::new("./.bench_db/monotree_hashmap_blake3")
}

fn init_monotree_rocksdb_blake3() -> Monotree<RocksDB, Blake3> {
    Monotree::new("./.bench_db/monotree_rocksdb_blake3")
}

fn init_monotree_sled_blake3() -> Monotree<Sled, Blake3> {
    Monotree::new("./.bench_db/monotree_sled_blake3")
}

// TODO    Add SHA2 flavor also
