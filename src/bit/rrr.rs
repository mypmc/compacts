include!(concat!(env!("OUT_DIR"), "/table.rs"));

use crate::bit::{cast, UnsignedInt};

// It is a good idea to choose `BLOCK_SIZE + 1` as a power of two,
// so that the bits that has size `CLASS_SIZE` can be fully used for bitpacking.
// e.g) 255: 8, 127: 7, 63: 6, 31: 5, 15: 4,

pub trait Code: UnsignedInt {
    /// The minimum bits size to represents class value of bits that has size `SIZE`.
    const CLASS: usize;

    /// The bit size of code.
    const SIZE: usize = (Self::BITS - 1) as usize;

    /// `MASK` is used to disable MSB (most significant bit).
    const MASK: Self;
}

macro_rules! implCode {
    ($(($type:ty, $class_size:expr)),*) => ($(
        impl Code for $type {
            const CLASS: usize = $class_size;
            const MASK: Self = (1 << Self::SIZE) - 1;
        }
    )*)
}

implCode!((u8, 3), (u16, 4), (u32, 5), (u64, 6), (u128, 7));
#[cfg(target_pointer_width = "32")]
implCode!((usize, 5));
#[cfg(target_pointer_width = "64")]
implCode!((usize, 6));

pub fn encode<C: Code>(mut code: C) -> (u64, C) {
    code &= C::MASK;

    let class = code.count1();
    let offset = {
        let mut c = cast::<u64, usize>(class);
        let mut o = 0;
        let mut j = 1;

        while 0 < c && c <= C::SIZE - j {
            if code.access(cast(C::SIZE - j)) {
                o += TABLE[C::SIZE - j][c];
                c -= 1;
            }
            j += 1;
        }
        o
    };

    (class, cast(offset))
}

pub fn decode<C: Code>(class: u64, offset: C) -> C {
    let mut code = C::ZERO;
    let mut c = cast::<u64, usize>(class);
    let mut o = offset;
    let mut j = 1usize;

    while c > 0 {
        if o >= cast(TABLE[C::SIZE - j][c]) {
            code.set1(cast::<usize, u64>(C::SIZE - j));
            o -= cast::<u128, C>(TABLE[C::SIZE - j][c]);
            c -= 1;
        }
        j += 1;
    }
    code
}
