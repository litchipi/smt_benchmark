use criterion::{criterion_group, criterion_main, Criterion};
use massa_smt_bench::lsmtree::add_lsmtree_benches;
use massa_smt_bench::monotree::add_monotree_benches;
// use massa_smt_bench::sparse_merkle_tree::add_sparse_merkle_tree_benches;

const SAMPLE_SIZE: usize = 1000;
const ELEMENTS_IN_TREE: usize = 10000;

// TODO    cw-merkle-tree
// TODO    lsmtree

pub fn all_frameworks(c: &mut Criterion) {
    add_monotree_benches(c, SAMPLE_SIZE, ELEMENTS_IN_TREE);
    // add_sparse_merkle_tree_benches(c, SAMPLE_SIZE, ELEMENTS_IN_TREE);
    add_lsmtree_benches(c, SAMPLE_SIZE, ELEMENTS_IN_TREE);
}

criterion_group!(benches, all_frameworks);
criterion_main!(benches);
