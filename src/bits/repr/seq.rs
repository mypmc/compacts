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

    pub fn number_of_runs(&self) -> usize {
        let mut runs = 0;
        match self.vector.len() {
            0 => 0,
            1 => 1,
            n => {
                for i in 1..n {
                    let prev = self.vector[i - 1];
                    let curr = self.vector[i];
                    if curr == prev + 1 {
                        // runlen += 1;
                    } else {
                        assert!(curr > prev);
                        runs += 1;
                        // runlen = 0;
                    }
                }
                runs + 1
            }
        }
    }
}

#[test]
fn test_number_of_runs() {
    macro_rules! seqblock {
        ( $( $bit:expr ),* ) => {
            {
                let mut seq = SeqBlock::new();
                $( seq.insert( $bit ); )*
                seq
            }
        }
    }

    let seq = seqblock!();
    assert_eq!(seq.number_of_runs(), 0);
    let seq = seqblock!(0, 1, 2, 3, 4, 5);
    assert_eq!(seq.number_of_runs(), 1);
    let seq = seqblock!(0, 1, 2, 4, 5);
    assert_eq!(seq.number_of_runs(), 2);
    let seq = seqblock!(0, 1, 2, 4, 5, 10, 20);
    assert_eq!(seq.number_of_runs(), 4);
    let seq = seqblock!(0, 1, 2, 4, 5, 10, 20, 40, 41, 42);
    assert_eq!(seq.number_of_runs(), 5);
}
