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

impl<W, T> WriteTo<W> for Vec<T>
where
    W: io::Write,
    T: WriteTo<W>,
{
    fn write_to(&self, w: &mut W) -> io::Result<()> {
        for t in self {
            t.write_to(w)?;
        }
        Ok(())
    }
}

impl<R, T> ReadFrom<R> for Vec<T>
where
    R: io::Read,
    T: ReadFrom<R>,
{
    fn read_from(&mut self, r: &mut R) -> io::Result<()> {
        for t in self {
            t.read_from(r)?;
        }
        Ok(())
    }
}

#[cfg(test)]
macro_rules! check {
    ( $v1:expr ) => {
        {
            let mut buf = Vec::with_capacity(8 * 8);
            assert!($v1.write_to(&mut buf).is_ok());
            let mut v2 = vec![0; 8];
            assert!(v2.read_from(&mut io::Cursor::new(buf)).is_ok());
            assert_eq!($v1, v2);
        }
    }
}

#[test]
fn read_write_vec() {
    check!(vec![1u16, 2, 4, 8, 16, 32, 64, 128]);
    check!(vec![1u32, 2, 4, 8, 16, 32, 64, 128]);
    check!(vec![1u64, 2, 4, 8, 16, 32, 64, 128]);
}
