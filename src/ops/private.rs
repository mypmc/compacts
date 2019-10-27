use super::*;

/// A trait to seal private trait.
pub trait Sealed {}

macro_rules! impl_Sealed {
    ( $( [ $($tts:tt)+ ] for $Type:ty; )* ) => {
        $( impl<$($tts)+> Sealed for $Type {} )*
    };
    ( $( $Type:ty ),* ) => {
        $( impl Sealed for $Type {} )*
    };
}

impl_Sealed!(u8, u16, u32, u64, u128, usize);
impl_Sealed!(i8, i16, i32, i64, i128, isize);
impl_Sealed!(RangeFull);

impl_Sealed!(
    [T: Sealed] for Range<T>;
    [T: Sealed] for RangeFrom<T>;
    [T: Sealed] for RangeTo<T>;
    [T: Sealed] for RangeInclusive<T>;
    [T: Sealed] for RangeToInclusive<T>;
    [T: Sealed] for (Bound<T>, Bound<T>);
);
