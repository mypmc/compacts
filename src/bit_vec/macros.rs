macro_rules! keypos {
    ( $bit:expr, $key:ident, $pos:ident ) => (
        use prim::Uint;
        let key  = $bit / <u64 as Uint>::WIDTH as u16;
        let $pos = $bit % <u64 as Uint>::WIDTH as u16;
        let $key = key as usize;
    );
}

macro_rules! bitmask {
    ( $bit:expr, $key:ident, $mask:ident ) => (
        keypos!($bit, $key, pos);
        let $mask = 1 << pos;
    );
}

macro_rules! delegate {
    ( $this: ident, $method: ident $(, $args: expr )* ) => {{
        use bit_vec::block::Block;
        match $this {
            Block::Sorted(vec) => vec.$method( $( $args ),* ),
            Block::Mapped(vec) => vec.$method( $( $args ),* ),
        }
    }};
    ( ref $this: ident, $method: ident $(, $args: expr )* ) => {{
        use bit_vec::block::Block;
        match $this {
            &Block::Sorted(ref vec) => vec.$method( $( $args ),* ),
            &Block::Mapped(ref vec) => vec.$method( $( $args ),* ),
        }
    }};
    ( ref mut $this: ident, $method: ident $(, $args: expr )* ) => {{
        use bit_vec::block::Block;
        match $this {
            &mut Block::Sorted(ref mut vec) => vec.$method( $( $args ),* ),
            &mut Block::Mapped(ref mut vec) => vec.$method( $( $args ),* ),
        }
    }}
}

macro_rules! extend_by_u16 {
    ( $bucket_or_block: expr, $iter: expr ) => {
        {
            let mut ones = 0;
            for item in $iter {
                if $bucket_or_block.insert(item) {
                    ones += 1;
                }
            }
            ones
        }
    }
}

macro_rules! bucket_foreach {
    ( $this:ident & $that:expr ) => {{
        bucket_foreach!($this, intersection_with, $that)
    }};
    ( $this:ident | $that:expr ) => {{
        bucket_foreach!($this, union_with,  $that)
    }};
    ( $this:ident - $that:expr ) => {{
        bucket_foreach!($this, difference_with, $that)
    }};
    ( $this:ident ^ $that:expr ) => {{
        bucket_foreach!($this, symmetric_difference_with, $that)
    }};

    ( $this:ident, $method:ident, $that:expr ) => {{
        use dict::Ranked;

        debug_assert!($this.vector.len() == $that.vector.len());

        let ref mut lhs = $this.vector;
        let ref rhs = $that.vector;
        $this.weight = {
            let mut new = 0;
            for (x, y) in lhs.iter_mut().zip(rhs.iter()) {
                (*x).$method(*y);
                new += x.count1();
            }
            new
        };
    }};
}

#[cfg(test)]
macro_rules! block {
    ( MIN_VEC; $rng: expr ) => {{
        let size = 0;
        let b = bucket!(u16; size as usize, $rng);
        Block::Sorted(b)
    }};
    ( MAX_VEC; $rng: expr ) => {{
        let size = THRESHOLD;
        let b = bucket!(u16; size as usize, $rng);
        Block::Sorted(b)
    }};
    ( MIN_MAP; $rng: expr ) => {{
        let size = THRESHOLD + 1;
        let b = bucket!(u64; size as usize, $rng);
        Block::Mapped(b)
    }};
    ( MAX_MAP; $rng: expr ) => {{
        let size = Block::CAPACITY - 1;
        let b = bucket!(u64; size as usize, $rng);
        Block::Mapped(b)
    }};
    ( VEC; $rng: expr ) => {{
        let size = $rng.gen_range(0, THRESHOLD);
        let b = bucket!(u16; size as usize, $rng);
        Block::Sorted(b)
    }};
    ( MAP; $rng: expr ) => {{
        let size = $rng.gen_range(THRESHOLD, Block::CAPACITY as usize);
        let b = bucket!(u64; size as usize, $rng);
        Block::Mapped(b)
    }};
}

#[cfg(test)]
macro_rules! bucket {
    ( u16; $size: expr, $rng: expr ) => {{
        let mut bucket = Bucket::<u16>::with_capacity($size);
        for _ in 0..$size {
            bucket.insert($rng.gen());
        }
        bucket
    }};
    ( u64; $size: expr, $rng: expr ) => {{
        let mut bucket = Bucket::<u64>::new();
        for _ in 0..$size {
            bucket.insert($rng.gen());
        }
        bucket
    }};
}

#[cfg(test)]
macro_rules! bench_block {
    ( $lhs:ident . $method:ident ( $rhs:ident ), $bench:expr ) => {{
        let (mut this, that) = setup_pair!($lhs, $rhs);
        $bench.iter( || this.$method(&that) );
    }};
}

#[cfg(test)]
macro_rules! bench_block_bitops {
    ( $lhs:ident & $rhs:ident , $bench:expr ) => {{
        let (this, that) = setup_pair!($lhs, $rhs);
        $bench.iter( || &this & &that );
    }};
    ( $lhs:ident | $rhs:ident , $bench:expr ) => {{
        let (this, that) = setup_pair!($lhs, $rhs);
        $bench.iter( || &this | &that );
    }};
    ( $lhs:ident - $rhs:ident , $bench:expr ) => {{
        let (this, that) = setup_pair!($lhs, $rhs);
        $bench.iter( || &this - &that );
    }};
    ( $lhs:ident ^ $rhs:ident , $bench:expr ) => {{
        let (this, that) = setup_pair!($lhs, $rhs);
        $bench.iter( || &this ^ &that );
    }};
}

#[cfg(test)]
macro_rules! setup_pair {
    ( $this:ident, $that:ident ) => {{
        let mut rng = rand::thread_rng();
        let lhs = block!($this; rng);
        let rhs = block!($that; rng);
        (lhs, rhs)
    }};
}

#[cfg(test)]
macro_rules! setup_test {
    ( $this:ident . $method:ident ( $that:ident ) ) => {{
        let (lhs, rhs) = setup_pair!($this, $that);
        let block = {
            let mut cloned = lhs.clone();
            cloned.$method(&rhs);
            cloned
        };
        (lhs, rhs, block)
    }};
}

#[cfg(test)]
macro_rules! bitops_test {
    ( $this:ident & $that:ident ) => {
        let (lhs, rhs, block) = setup_test!($this.intersection_with($that));
        for bit in &block {
            assert!(lhs.contains(bit) && rhs.contains(bit),
                    "{lhs:?} AND {rhs:?}: block={block:?}",
                    lhs = lhs, rhs = rhs, block = block);
        }
        let expect = {
            use bit_vec::pairwise::intersection;
            let pair = intersection(lhs.iter(), rhs.iter());
            pair.collect::<Block>().count1()
        };
        assert!(block.count1() == expect,
                "{lhs:?} AND {rhs:?}: got={got:?} want={want:?} ",
                lhs = lhs, rhs = rhs, got = block.count1(), want = expect);
    };

    ( $this:ident | $that:ident ) => {
        let (lhs, rhs, block) = setup_test!($this.union_with($that));
        for bit in &block {
            assert!(lhs.contains(bit) || rhs.contains(bit),
                    "{lhs:?} OR {rhs:?}: block={block:?}",
                    lhs = lhs, rhs = rhs, block = block);
        }
        let expect = {
            use bit_vec::pairwise::union;
            let pair = union(lhs.iter(), rhs.iter());
            pair.collect::<Block>().count1()
        };
        assert!(block.count1() == expect,
                "{lhs:?} OR {rhs:?}: got={got:?} want={want:?}",
                lhs = lhs, rhs = rhs, got = block.count1(), want = expect);
    };

    ( $this:ident - $that:ident ) => {
        let (lhs, rhs, block) = setup_test!($this.difference_with($that));

        for bit in &block {
            assert!(lhs.contains(bit) && !rhs.contains(bit),
                    "{lhs:?} - {rhs:?}: block={block:?}",
                    lhs = lhs, rhs = rhs, block=block);
        }
        let expect = {
            use bit_vec::pairwise::difference;
            let pair = difference(lhs.iter(), rhs.iter());
            pair.collect::<Block>().count1()
        };
        assert!(block.count1() == expect,
                "{lhs:?} - {rhs:?}: got={got:?} want={want:?}",
                lhs = lhs, rhs = rhs, got = block.count1(), want = expect);
    };

    ( $this:ident ^ $that:ident ) => {
        let (lhs, rhs, block) = setup_test!($this.symmetric_difference_with($that));

        for bit in &block {
            assert!(!(lhs.contains(bit) && rhs.contains(bit)),
                    "{lhs:?} XOR {rhs:?}: block={block:?}",
                    lhs = lhs, rhs = rhs, block=block);
        }
        let expect = {
            use bit_vec::pairwise::symmetric_difference;
            let pair = symmetric_difference(lhs.iter(), rhs.iter());
            pair.collect::<Block>().count1()
        };
        assert!(block.count1() == expect,
                "{lhs:?} XOR {rhs:?}: got={got:?} want={want:?}",
                lhs = lhs, rhs = rhs, got = block.count1(), want = expect);
    };
}
