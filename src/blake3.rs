use lsmtree::digest::generic_array::GenericArray;

#[derive(Debug, Default, Clone)]
pub struct Blake3SmtHasher(blake3::Hasher);

// MONOTREE

impl monotree::hasher::Hasher for Blake3SmtHasher {
    fn new() -> Self {
        Blake3SmtHasher(blake3::Hasher::new())
    }

    fn digest(&self, bytes: &[u8]) -> monotree::Hash {
        let mut hasher = blake3::Hasher::new();
        hasher.update(bytes);
        hasher.finalize().into()
    }
}

// SPARSE MERKLE TREE

impl sparse_merkle_tree::traits::Hasher for Blake3SmtHasher {
    fn write_h256(&mut self, h: &sparse_merkle_tree::H256) {
        self.0.update(h.as_slice());
    }

    fn write_byte(&mut self, b: u8) {
        self.0.update(&[b][..]);
    }

    fn finish(self) -> sparse_merkle_tree::H256 {
        let hash: [u8; 32] = self.0.finalize().into();
        hash.into()
    }
}

// LSMTREE

impl lsmtree::digest::OutputSizeUser for Blake3SmtHasher {
    type OutputSize = lsmtree::digest::typenum::U32;
}

impl lsmtree::digest::Digest for Blake3SmtHasher {
    fn new() -> Self {
        Blake3SmtHasher(blake3::Hasher::new())
    }

    fn new_with_prefix(_: impl AsRef<[u8]>) -> Self {
        todo!()
    }

    fn update(&mut self, data: impl AsRef<[u8]>) {
        self.0.update(data.as_ref());
    }

    fn chain_update(self, _: impl AsRef<[u8]>) -> Self {
        todo!()
    }

    fn finalize(self) -> lsmtree::digest::Output<Self> {
        GenericArray::clone_from_slice(self.0.finalize().as_bytes())
    }

    fn finalize_into(self, _: &mut lsmtree::digest::Output<Self>) {
        unreachable!()
    }

    fn finalize_reset(&mut self) -> lsmtree::digest::Output<Self> {
        unreachable!()
    }

    fn finalize_into_reset(&mut self, _: &mut lsmtree::digest::Output<Self>) {
        unreachable!()
    }

    fn reset(&mut self) {
        unreachable!()
    }

    fn output_size() -> usize {
        <Self as lsmtree::digest::OutputSizeUser>::output_size()
    }

    fn digest(data: impl AsRef<[u8]>) -> lsmtree::digest::Output<Self> {
        let mut h = Self::new();
        h.update(data);
        h.finalize()
    }
}
