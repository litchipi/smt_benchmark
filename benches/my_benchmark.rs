use criterion::{criterion_group, criterion_main, Criterion};
use massa_smt_bench::monotree::add_monotree_benches;

const SAMPLE_SIZE: usize = 100;
const ELEMENTS_IN_TREE: usize = 1000;

// TODO    Sparse-merkle-tree
// TODO    cw-merkle-tree
// TODO    lsmtree

pub fn all_frameworks(c: &mut Criterion) {
    add_monotree_benches(c, SAMPLE_SIZE, ELEMENTS_IN_TREE);
}

criterion_group!(benches, all_frameworks);
criterion_main!(benches);
