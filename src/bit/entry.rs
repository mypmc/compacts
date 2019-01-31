use crate::bit::{self, ops::FiniteBits, UnsignedInt};

/// `Entry` holds value `V` and its index `K`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry<K: UnsignedInt, V> {
    pub(crate) index: K,
    pub(crate) value: V,
}

impl<K: UnsignedInt, V> Entry<K, V> {
    pub fn new(index: K, value: V) -> Self {
        Self { index, value }
    }

    pub fn index(&self) -> &K {
        &self.index
    }
    pub fn value(&self) -> &V {
        &self.value
    }

    pub(crate) fn find(slice: &[Self], index: &K) -> Result<usize, usize> {
        slice.binary_search_by_key(index, |entry| entry.index)
    }

    pub(crate) fn potential_bits() -> u64
    where
        V: FiniteBits,
    {
        // (1<<K::BITS) * V::BITS
        1u64.checked_shl(K::BITS as u32)
            .and_then(|len| len.checked_mul(V::BITS))
            .map_or(bit::MAX, |bits| std::cmp::min(bits, bit::MAX))
    }
}
