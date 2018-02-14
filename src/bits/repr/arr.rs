use super::ArrBlock;
use super::PopCount;

impl PartialEq for ArrBlock {
    fn eq(&self, that: &ArrBlock) -> bool {
        let length_test = self.bitmap.len() == that.bitmap.len();
        let weight_test = self.weight == that.weight;
        let boxarr_test = self.bitmap
            .iter()
            .zip(that.bitmap.iter())
            .all(|(v1, v2)| v1 == v2);
        length_test && weight_test && boxarr_test
    }
}
impl Eq for ArrBlock {}

impl Default for ArrBlock {
    fn default() -> Self {
        let weight = 0;
        let bitmap = Box::new([0; 1024]);
        // let bitmap = [0; 1024];
        ArrBlock { weight, bitmap }
    }
}

impl ArrBlock {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn weight(&self) -> u32 {
        self.weight
    }
    pub fn bitmap(&self) -> &[u64] {
        &*self.bitmap
    }

    #[inline]
    pub fn check_enabled(&self, i: usize, mask: u64) -> bool {
        self.bitmap
            .get(i)
            .map(|&word| word & mask != 0)
            .unwrap_or_default()
    }

    #[inline]
    pub fn contains(&self, bit: u16) -> bool {
        let (i, p) = divrem!(bit, 64);
        self.check_enabled(i, 1 << p)
    }

    #[inline]
    pub fn insert(&mut self, bit: u16) -> bool {
        let (i, p) = divrem!(bit, 64);
        if self.check_enabled(i, 1 << p) {
            true
        } else {
            self.bitmap[i] |= 1 << p;
            self.weight += 1;
            false
        }
    }

    #[inline]
    pub fn remove(&mut self, bit: u16) -> bool {
        let (i, p) = divrem!(bit, 64);
        if self.check_enabled(i, 1 << p) {
            self.bitmap[i] &= !(1 << p);
            self.weight -= 1;
            true
        } else {
            false
        }
    }

    pub fn number_of_runs(&self) -> usize {
        if self.weight == 0 {
            0
        } else {
            let mut runs = 0;
            let mut next = self.bitmap[0];

            for i in 0..(self.bitmap.len() - 1) {
                let word = next;
                next = self.bitmap[i + 1];
                let mut rb = !word;
                rb &= (word << 1) + (word >> 63);
                rb &= !next;
                let popc: u64 = rb.count1();
                runs += popc as usize;
            }

            let word = next;
            let popc: u64 = (!word & (word << 1)).count1();
            runs += popc as usize;
            if (word & 0x8000_0000_0000_0000) != 0 {
                runs += 1;
            }

            runs
        }
    }
}

#[test]
fn test_number_of_runs() {
    macro_rules! arrblock {
        ( $( $bit:expr ),* ) => {
            {
                let mut arr = ArrBlock::new();
                $( arr.insert( $bit ); )*
                arr
            }
        }
    }

    let arr = arrblock!();
    assert_eq!(arr.number_of_runs(), 0);
    let arr = arrblock!(0, 1, 2, 3, 4, 5);
    assert_eq!(arr.number_of_runs(), 1);
    let arr = arrblock!(0, 1, 2, 4, 5);
    assert_eq!(arr.number_of_runs(), 2);
    let arr = arrblock!(0, 1, 2, 4, 5, 10, 20);
    assert_eq!(arr.number_of_runs(), 4);
}
