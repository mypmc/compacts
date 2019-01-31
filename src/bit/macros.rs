macro_rules! generate_rrr_mod {
    ($file:expr, $data:ty, $block_size:expr, $class_size:expr) => {
        // It is a good idea to choose `size + 1` as a power of two,
        // so that the bits for `class` can be fully used (bitpacking).
        // e.g) 255: 8, 127: 7, 63: 6, 31: 5, 15: 4,

        include!(concat!(env!("OUT_DIR"), $file));

        /// the bit size of block.
        pub const BLOCK_SIZE: usize = $block_size;

        /// minimum bits size to represents class value.
        pub const CLASS_SIZE: usize = $class_size;

        pub fn encode(mut data: $data) -> (u32, $data) {
            data &= (1 << BLOCK_SIZE) - 1;

            let class = data.count_ones();
            let offset = {
                let mut c = class as usize;
                let mut o = 0;
                let mut j = 1;

                while 0 < c && c <= BLOCK_SIZE - j {
                    if data & (1 << (BLOCK_SIZE - j)) != 0 {
                        o += TABLE[BLOCK_SIZE - j][c];
                        c -= 1;
                    }
                    j += 1;
                }
                o
            };
            (class, offset)
        }

        pub fn decode(class: u32, offset: $data) -> $data {
            let mut data = 0;
            let mut c = class as usize;
            let mut o = offset;
            let mut j = 1usize;

            while c > 0 {
                if o >= TABLE[BLOCK_SIZE - j][c] {
                    data |= 1 << (BLOCK_SIZE - j);
                    o -= TABLE[BLOCK_SIZE - j][c];
                    c -= 1;
                }
                j += 1;
            }
            data
        }
    };
}
