use super::{SeqBlock, SEQ_MAX_LEN};

impl SeqBlock {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(cap: usize) -> Self {
        let bounded = if cap <= SEQ_MAX_LEN { cap } else { SEQ_MAX_LEN };
        let vector = Vec::with_capacity(bounded);
        SeqBlock { vector }
    }

    pub fn vector(&self) -> &[u16] {
        &self.vector
    }

    #[inline]
    pub fn search(&self, bit: &u16) -> Result<usize, usize> {
        self.vector.binary_search(bit)
    }

    #[inline]
    pub fn contains(&self, bit: u16) -> bool {
        self.search(&bit).is_ok()
    }

    #[inline]
    pub fn insert(&mut self, bit: u16) -> bool {
        self.search(&bit)
            .map_err(|i| self.vector.insert(i, bit))
            .is_ok()
    }

    #[inline]
    pub fn remove(&mut self, bit: u16) -> bool {
        self.search(&bit).map(|i| self.vector.remove(i)).is_ok()
    }
}
