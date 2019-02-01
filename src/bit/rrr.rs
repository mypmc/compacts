//! # Reference
//!
//! Succinct indexable dictionaries with applications to encoding k-ary trees and multisets
//!
//! https://portal.acm.org/citation.cfm?id=545411
//!

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

//// read bits in [i, j)
//pub fn read_code<U>(slice: &[u8], i: u64, j: u64) -> U
//where
//    U: UnsignedInt,
//{
//    assert!(i < j && j - i <= U::BITS && i < slice.bits() && j <= slice.bits());

//    let j = j - 1; // make inclusive

//    let (head_index, head_offset) = divmod::<usize>(i, u8::BITS);
//    let (last_index, last_offset) = divmod::<usize>(j, u8::BITS);

//    if head_index == last_index {
//        slice[head_index].read(head_offset..last_offset + 1)
//    } else {
//        // head_index < last_index

//        // returning value
//        let mut out = U::ZERO;
//        // how many bits do we have read?
//        let mut len = 0;

//        out |= slice[head_index].read::<U>(head_offset..u8::BITS);
//        len += u8::BITS - head_offset;

//        for &word in &slice[(head_index + 1)..last_index] {
//            out |= cast::<u8, U>(word).shiftl(len);
//            len += u8::BITS;
//        }

//        let last = slice[last_index].read::<U>(0..last_offset + 1);
//        // debug_assert_eq!(
//        //     cast::<u8, u64>(last),
//        //     cast::<U, u64>((cast::<u8, U>(last) << cast(len)) >> cast(len))
//        // );
//        //
//        // last need to be shifted to left by `len`
//        out | last.shiftl(len)
//    }
//}

pub fn encode<C: Code>(code: C) -> (u64, C) {
    let code = code & C::MASK;

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
