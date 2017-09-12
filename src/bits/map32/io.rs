use std::io;

// use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use bits;
// use dict::PopCount;

use super::Map;

// `WriteTo`, `ReadFrom` should follows this spec
// https://github.com/RoaringBitmap/RoaringFormatSpec

// const SERIAL_COOKIE_NO_RLE: u32 = 12_346;
// const SERIAL_COOKIE_HAS_RLE: u16 = 12_347;
// const NO_OFFSET_THRESHOLD: u8 = 4;

impl<W: io::Write> bits::WriteTo<W> for Map {
    fn write_to(&self, w: &mut W) -> io::Result<()> {
        // w.write_u32::<LittleEndian>(SERIAL_COOKIE_NO_RLE)?;
        // w.write_u32::<LittleEndian>(self.map16s.len() as u32)?;

        // for (&key, map) in &self.map16s {
        //     w.write_u16::<LittleEndian>(key)?;
        //     w.write_u16::<LittleEndian>((map.count1() - 1) as u16)?;
        // }

        // let mut offset = 8 + 8 * self.map16s.len() as u32;
        // for (&key, map) in &self.map16s {
        //     w.write_u32::<LittleEndian>(offset)?;
        //     match map.block {
        //         Block::Seq16(ref b) => {
        //             offset += (mem::size_of::<u16>() * b.vector.len()) as u32;
        //         }
        //         Block::Arr64(..) => {
        //             offset += (mem::size_of::<u64>() as u32) * 1024;
        //         }
        //         Block::Run16(..) => {
        //             //
        //             unimplemented!();
        //         }
        //     }
        // }

        // for (&key, map) in &self.map16s {
        //     match map.block {
        //         Block::Seq16(ref b) => for &v in &b.vector {
        //             w.write_u16::<LittleEndian>(v)?;
        //         },
        //         Block::Arr64(ref b) => for &v in &b.vector {
        //             w.write_u64::<LittleEndian>(v)?;
        //         },
        //         Block::Run16(..) => {
        //             //
        //             unimplemented!();
        //         }
        //     }
        // }

        unimplemented!();
    }
}

impl<R: io::Read> bits::ReadFrom<R> for Map {
    fn read_from(&mut self, r: &mut R) -> io::Result<()> {
        unimplemented!()
    }
}
