use std::io;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

/// Trait for write content to W.
pub trait WriteTo<W: io::Write> {
    fn write_to(&self, w: &mut W) -> io::Result<()>;
}

/// Trait for read content from R.
pub trait ReadFrom<R: io::Read> {
    fn read_from(&mut self, r: &mut R) -> io::Result<()>;
}

pub fn read_from<R, T>(r: &mut R) -> io::Result<T>
where
    R: io::Read,
    T: Default + ReadFrom<R>,
{
    let mut data = <T as Default>::default();
    data.read_from(r)?;
    Ok(data)
}

macro_rules! impl_word_WriteTo {
    ( $( ( $typ:ty, $method:ident ) ),* ) => ($(
        impl<W: io::Write> WriteTo<W> for $typ {
            fn write_to(&self, w: &mut W) -> io::Result<()> {
                w.$method::<LittleEndian>(*self)
            }
        }
    )*)
}

macro_rules! impl_word_ReadFrom {
    ( $( ( $typ:ty, $method:ident ) ),* ) => ($(
        impl<R: io::Read> ReadFrom<R> for $typ {
            fn read_from(&mut self, r: &mut R) -> io::Result<()> {
                *self = r.$method::<LittleEndian>()?;
                Ok(())
            }
        }
    )*)
}

impl_word_WriteTo!((u16, write_u16), (u32, write_u32), (u64, write_u64));
impl_word_ReadFrom!((u16, read_u16), (u32, read_u32), (u64, read_u64));
