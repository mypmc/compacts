//! Module `rrr`

use std::marker::PhantomData;

use crate::num::{cast, mask1, Word};

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rrr<W> {
    code_size: usize,
    _word: PhantomData<W>,
}

impl<W: Word> Default for Rrr<W> {
    fn default() -> Self {
        Self {
            code_size: W::BITS - 1,
            _word: PhantomData,
        }
    }
}

pub(crate) mod rrr_static {
    /// Pre computed table of the number of combinations.
    ///
    /// See `build.rs` for more details.
    #[allow(clippy::unreadable_literal)]
    pub static TABLE: [[u128; 127]; 127] = include!(concat!(env!("OUT_DIR"), "/table.rs"));
}

#[doc(hidden)]
impl<W: Word> Rrr<W> {
    pub fn code_size(n: usize) -> Option<Self> {
        if n > W::BITS {
            None
        } else {
            Some(Self {
                code_size: n,
                _word: PhantomData,
            })
        }
    }

    pub fn encode(self, word: W) -> (usize, W) {
        let code_size = self.code_size;
        assert!(code_size <= W::BITS);

        let code = word & mask1::<W>(code_size);

        let class = code.count1();
        let offset = {
            let mut c = class;
            let mut o = 0;
            let mut j = 1;

            while 0 < c && c <= code_size - j {
                if code.bit(code_size - j) {
                    o += rrr_static::TABLE[code_size - j][c];
                    c -= 1;
                }
                j += 1;
            }
            cast(o)
        };
        (class, offset)
    }

    pub fn decode(self, class: usize, offset: W) -> W {
        let code_size = self.code_size;
        assert!(code_size <= W::BITS);

        let mut code = W::_0;
        let mut c = class;
        let mut o = offset;
        let mut j = 1usize;

        while c > 0 {
            let comb = cast(rrr_static::TABLE[code_size - j][c]);
            if o >= comb {
                code.put1(code_size - j);
                o -= comb;
                c -= 1;
            }
            j += 1;
        }
        code
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::*;
    use quickcheck::quickcheck;

    quickcheck! {
        fn rrr32(code: u32) -> bool {
            let (class, offset) = Rrr::<u32>::default().encode(code);
            let got = Rrr::<u32>::default().decode(class, offset);
            got == code.getn::<u32>(0, Bits::size(&code) - 1)
        }

        fn rrr64(code: u64) -> bool {
            let (class, offset) = Rrr::<u64>::default().encode(code);
            let got = Rrr::<u64>::default().decode(class, offset);
            got == code.getn::<u64>(0, Bits::size(&code) - 1)
        }

        fn rrr128(code: u128) -> bool {
            let (class, offset) = Rrr::<u128>::default().encode(code);
            let got = Rrr::<u128>::default().decode(class, offset);
            got == code.getn::<u128>(0, Bits::size(&code) - 1)
        }
    }
}
