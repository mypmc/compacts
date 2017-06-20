use super::BitVec;

#[derive(Clone, Debug)]
pub struct Stats {
    pub count: u64,
    pub block: usize,

    pub of_vec16: BlockStats,
    pub of_vec64: BlockStats,
    pub of_rle16: BlockStats,
}

#[derive(Clone, Debug)]
pub struct BlockStats {
    pub count: u64,
    pub bytes: u64,
}

impl BitVec {
    pub fn stats(&self) -> Stats {
        let mut stats = Stats {
            count: self.count_ones(),
            block: self.blocks.len(),
            of_vec16: BlockStats { count: 0, bytes: 0 },
            of_vec64: BlockStats { count: 0, bytes: 0 },
            of_rle16: BlockStats { count: 0, bytes: 0 },
        };
        for block in self.blocks.values() {
            match **block {
                super::Block::Seq16(ref b) => {
                    stats.of_vec16.count += b.count_ones() as u64;
                    stats.of_vec16.bytes += b.mem() as u64;
                }
                super::Block::Seq64(ref b) => {
                    stats.of_vec64.count += b.count_ones() as u64;
                    stats.of_vec64.bytes += b.mem() as u64;
                }
                super::Block::Rle16(ref b) => {
                    stats.of_rle16.count += b.count_ones() as u64;
                    stats.of_rle16.bytes += b.mem() as u64;
                }
            }
        }
        stats
    }
}
