use std::io;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use bits::PopCount;
use super::{ArrBlock, RunBlock, SeqBlock};

impl SeqBlock {
    pub fn write_to<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        for &bit in &self.vector {
            w.write_u16::<LittleEndian>(bit)?;
        }
        Ok(())
    }

    pub fn read_from<R>(r: &mut R, len: usize) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut vector = vec![0; len];
        for bit in &mut vector {
            *bit = r.read_u16::<LittleEndian>()?;
        }
        Ok(SeqBlock { vector })
    }
}

impl ArrBlock {
    pub fn write_to<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        for &bit in self.bitmap.iter() {
            w.write_u64::<LittleEndian>(bit)?;
        }
        Ok(())
    }

    pub fn read_from<R>(r: &mut R) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut this = ArrBlock::default();
        for bit in this.bitmap.iter_mut() {
            *bit = r.read_u64::<LittleEndian>()?;
            this.weight += <u64 as PopCount<u32>>::count1(bit);
        }
        Ok(this)
    }
}

// `RunBlock` is serialized as a 16-bit integer indicating the number of runs,
// followed by a pair of 16-bit values for each run.
// Runs are non-overlapping and sorted.
// For example, the values `[11,12,13,14,15]` will be encoded to `11,4`
// where 4 means that beyond 11 itself.
//
// Example:
//    `[(1,3),(20,0),(31,2)]` => `[1, 2, 3, 4, 20, 31, 32, 33]`

impl RunBlock {
    pub fn write_to<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        w.write_u16::<LittleEndian>(self.ranges.len() as u16)?;
        for rg in &self.ranges {
            w.write_u16::<LittleEndian>(rg.start)?;
            w.write_u16::<LittleEndian>(rg.end - rg.start)?;
        }
        Ok(())
    }

    // Resize automatically.
    pub fn read_from<R>(r: &mut R) -> io::Result<Self>
    where
        R: io::Read,
    {
        let runs = r.read_u16::<LittleEndian>()?;

        let mut weight = 0;
        let mut ranges = vec![0..=0; runs as usize];

        for rg in &mut ranges {
            let s = r.read_u16::<LittleEndian>()?;
            let o = r.read_u16::<LittleEndian>()?;
            *rg = s..=(s + o);
            weight += u32::from(o) + 1;
        }

        Ok(RunBlock { weight, ranges })
    }
}
