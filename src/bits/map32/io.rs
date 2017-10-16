use std::{fmt, io, mem};

// use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use bits::{self, PopCount};
use io::{read_from, ReadFrom, WriteTo};

// https://github.com/RoaringBitmap/RoaringFormatSpec

const SERIAL_COOKIE: u16 = 12_347; // `Seq16`, `Arr64` and `Run16`
const SERIAL_NO_RUN: u32 = 12_346; // `Seq16` and `Arr64`

// The cookie header spans either 32 bits or 64 bits.
//
// If the first 32 bits have the value `SERIAL_NO_RUN`,
// then there is no `Run16` block in `Map`.
// the next 32 bits are used to store the number of blocks.
// If the bitmap is empty (i.e., it has no container),
// then you should choose this cookie header.
//
// If the 16 least significant bits of the 32-bit cookie have the value `SERIAL_COOKIE`,
// the 16 most significant bits of the 32-bit cookie are used to store
// the number of blocks minus 1.
// That is, if you shift right by 16 the cookie and add 1, you get the number of blocks.
//
// Then we store `RunIndex` following the initial 32 bits,
// as a bitset to indicate whether each of the blocks is a `Run16` or not.
//
// The LSB of the first byte corresponds to the first stored blocks and so forth.
// Thus if follows that the least significant 16 bits of the first 32 bits
// of a serialized bitmaps should either have the value `SERIAL_NO_RUN`
// or the value SERIAL_COOKIE. In other cases, we should abort the decoding.
//
// After scanning the cookie header, we know how many containers are present in the bitmap.

const NO_OFFSET_THRESHOLD: u8 = 4;

struct RunIndex {
    hasrun: bool,
    bitmap: Vec<u8>,
}
impl fmt::Debug for RunIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RunIndex(")?;
        for byte in &self.bitmap {
            write!(f, "{:08b}", byte)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl RunIndex {
    fn empty(&self) -> bool {
        !self.hasrun
    }
    fn bytes(&self) -> &[u8] {
        &self.bitmap
    }
}

impl bits::Map {
    fn sizeof_run_index(&self) -> usize {
        (self.blocks.len() + 7) / 8
    }

    fn run_index(&self) -> RunIndex {
        let mut hasrun = false;
        let mut bitmap = vec![0u8; self.sizeof_run_index()];
        for (i, b) in self.blocks.values().enumerate() {
            if let bits::Block::Run16(_) = *b {
                hasrun = true;
                bitmap[i / 8] |= 1 << (i % 8);
            }
        }
        RunIndex { hasrun, bitmap }
    }
}

impl<W: io::Write> WriteTo<W> for bits::Map {
    fn write_to(&self, w: &mut W) -> io::Result<()> {
        let runidx = self.run_index();

        let (sizeof_cookie, sizeof_runidx) = if runidx.empty() {
            (2 * mem::size_of::<u32>(), 0)
        } else {
            (2 * mem::size_of::<u16>(), runidx.bytes().len())
        };

        let sizeof_header = 2 * mem::size_of::<u16>() * self.blocks.len();
        let sum_sizeof = sizeof_cookie + sizeof_runidx + sizeof_header;

        // serial cookie
        if runidx.empty() {
            SERIAL_NO_RUN.write_to(w)?;
            (self.blocks.len() as u32).write_to(w)?;
        } else {
            SERIAL_COOKIE.write_to(w)?;
            ((self.blocks.len() - 1) as u16).write_to(w)?;

            w.write_all(runidx.bytes())?;
        };

        // header
        for (&key, block) in &self.blocks {
            let pop_decr = (block.count1() - 1) as u16;
            key.write_to(w)?;
            pop_decr.write_to(w)?;
        }

        if runidx.empty() || self.blocks.len() >= NO_OFFSET_THRESHOLD as usize {
            // offset
            let mut offset = sum_sizeof + 2 * mem::size_of::<u16>() * self.blocks.len();
            for b in self.blocks.values() {
                (offset as u32).write_to(w)?;
                let pop = b.count1();
                match *b {
                    bits::Block::Seq16(_) => {
                        assert!(pop as usize <= bits::Seq16::THRESHOLD);
                        offset += mem::size_of::<u16>() * pop as usize;
                    }
                    bits::Block::Arr64(_) => {
                        assert!(pop as usize > bits::Seq16::THRESHOLD);
                        offset += bits::Block::CAPACITY / 8;
                    }
                    bits::Block::Run16(ref run) => {
                        offset += mem::size_of::<u16>();
                        offset += 2 * mem::size_of::<u16>() * run.ranges.len();
                    }
                }
            }
        }

        // TODO: Fix Block's WriteTo implementation
        // Write an optimized block (clone if it should do so),
        // so that the above assertions can be removed.

        for b in self.blocks.values() {
            match *b {
                bits::Block::Seq16(ref seq) => seq.write_to(w)?,
                bits::Block::Arr64(ref arr) => arr.write_to(w)?,
                bits::Block::Run16(ref run) => run.write_to(w)?,
            }
        }

        Ok(())
    }
}

fn read_header<R: io::Read>(r: &mut R, size: usize) -> io::Result<Vec<(u16, u32)>> {
    let mut vec = Vec::with_capacity(size);
    for _ in 0..size {
        let key = read_from::<R, u16>(r)?;
        let pop = read_from::<R, u16>(r)?;
        vec.push((key, u32::from(pop) + 1));
    }
    // vec is sorted?
    Ok(vec)
}

fn discard_offset<R: io::Read>(r: &mut R, size: usize) -> io::Result<()> {
    let mut _offset = 0u32;
    for _ in 0..size {
        _offset.read_from(r)?;
    }
    Ok(())
}

impl<R: io::Read> ReadFrom<R> for bits::Map {
    fn read_from(&mut self, r: &mut R) -> io::Result<()> {
        self.clear();

        match read_from::<R, u32>(r)? {
            cookie if cookie == SERIAL_NO_RUN => {
                let block_len = read_from::<R, u32>(r)? as usize;
                // eprintln!("blocks={:?}", block_len);
                let header = read_header(r, block_len)?;
                // eprintln!("header={:?}", header);

                discard_offset(r, block_len)?;

                for i in 0..block_len {
                    let key = header[i].0;
                    let pop = header[i].1 as usize;

                    let block = if pop > bits::Seq16::THRESHOLD {
                        let block = read_from::<R, bits::Arr64>(r)?;
                        bits::Block::from(block)
                    } else {
                        let mut block = bits::Seq16 {
                            vector: vec![0; pop],
                        };
                        block.read_from(r)?;
                        bits::Block::from(block)
                    };
                    self.blocks.insert(key, block);
                }
                Ok(())
            }

            cookie if cookie & 0x0000FFFF == u32::from(SERIAL_COOKIE) => {
                let block_len = (cookie.wrapping_shr(16) + 1) as usize;
                let bytes_len = (block_len + 7) / 8;

                // eprintln!("blocks={} bytes={}", block_len, bytes_len);

                let hasrun = true;
                let bitmap = {
                    let mut buf = vec![0; bytes_len];
                    r.read_exact(&mut buf)?;
                    buf
                };
                let runidx = RunIndex { hasrun, bitmap };
                let header = read_header(r, block_len)?;

                // eprintln!("header={:?} {:?}", header, runidx);

                if runidx.empty() || block_len >= NO_OFFSET_THRESHOLD as usize {
                    discard_offset(r, block_len)?;
                }

                for i in 0..block_len {
                    let key = header[i].0;
                    let pop = header[i].1 as usize;

                    let block = if runidx.bitmap[i / 8] & (1 << (i % 8)) > 0 {
                        let block = read_from::<R, bits::Run16>(r)?;
                        bits::Block::from(block)
                    } else if pop > bits::Seq16::THRESHOLD {
                        let block = read_from::<R, bits::Arr64>(r)?;
                        bits::Block::from(block)
                    } else {
                        let mut block = bits::Seq16 {
                            vector: vec![0; pop],
                        };
                        block.read_from(r)?;
                        bits::Block::from(block)
                    };
                    self.blocks.insert(key, block);
                }
                Ok(())
            }

            x => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unexpected cookie value: {}", x),
            )),
        }
    }
}
