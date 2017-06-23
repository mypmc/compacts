#[cfg(test)]
macro_rules! make_block {
    ( MIN_VEC; $rng: expr ) => {{
        let size = 0;
        let b = random_block!(u16; size as usize, $rng);
        Seq16(b)
    }};
    ( MAX_VEC; $rng: expr ) => {{
        let size = block::Seq16::THRESHOLD;
        let b = random_block!(u16; size as usize, $rng);
        Seq16(b)
    }};
    ( MIN_MAP; $rng: expr ) => {{
        let size = block::Seq16::THRESHOLD + 1;
        let b = random_block!(u64; size as usize, $rng);
        Seq64(b)
    }};
    ( MAX_MAP; $rng: expr ) => {{
        let size = Vec16::CAPACITY - 1;
        let b = random_block!(u64; size as usize, $rng);
        Seq64(b)
    }};
    ( VEC; $rng: expr ) => {{
        let size = $rng.gen_range(0, block::Seq16::THRESHOLD);
        let b = random_block!(u16; size as usize, $rng);
        Seq16(b)
    }};
    ( MAP; $rng: expr ) => {{
        let size = $rng.gen_range(block::Seq16::THRESHOLD, Vec16::CAPACITY as usize);
        let b = random_block!(u64; size as usize, $rng);
        Seq64(b)
    }};
}

#[cfg(test)]
macro_rules! random_block {
    ( u16; $size: expr, $rng: expr ) => {{
        let mut block = block::Seq16::with_capacity($size);
        for _ in 0..$size {
            block.insert($rng.gen());
        }
        block
    }};
    ( u64; $size: expr, $rng: expr ) => {{
        let mut block = block::Seq64::new();
        for _ in 0..$size {
            block.insert($rng.gen());
        }
        block
    }};
}

#[cfg(test)]
macro_rules! setup_pair {
    ( $this:ident, $that:ident ) => {{
        let mut rng = rand::thread_rng();
        let lhs = make_block!($this; rng);
        let rhs = make_block!($that; rng);
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
            use pairwise::intersection;
            let pair = intersection(lhs.iter(), rhs.iter());
            pair.collect::<Vec16>().count_ones()
        };
        assert!(block.count_ones() == expect,
                "{lhs:?} AND {rhs:?}: got={got:?} want={want:?} ",
                lhs = lhs, rhs = rhs, got = block.count_ones(), want = expect);
    };

    ( $this:ident | $that:ident ) => {
        let (lhs, rhs, block) = setup_test!($this.union_with($that));
        for bit in &block {
            assert!(lhs.contains(bit) || rhs.contains(bit),
                    "{lhs:?} OR {rhs:?}: block={block:?}",
                    lhs = lhs, rhs = rhs, block = block);
        }
        let expect = {
            use pairwise::union;
            let pair = union(lhs.iter(), rhs.iter());
            pair.collect::<Vec16>().count_ones()
        };
        assert!(block.count_ones() == expect,
                "{lhs:?} OR {rhs:?}: got={got:?} want={want:?}",
                lhs = lhs, rhs = rhs, got = block.count_ones(), want = expect);
    };

    ( $this:ident - $that:ident ) => {
        let (lhs, rhs, block) = setup_test!($this.difference_with($that));

        for bit in &block {
            assert!(lhs.contains(bit) && !rhs.contains(bit),
                    "{lhs:?} - {rhs:?}: block={block:?}",
                    lhs = lhs, rhs = rhs, block=block);
        }
        let expect = {
            use pairwise::difference;
            let pair = difference(lhs.iter(), rhs.iter());
            pair.collect::<Vec16>().count_ones()
        };
        assert!(block.count_ones() == expect,
                "{lhs:?} - {rhs:?}: got={got:?} want={want:?}",
                lhs = lhs, rhs = rhs, got = block.count_ones(), want = expect);
    };

    ( $this:ident ^ $that:ident ) => {
        let (lhs, rhs, block) = setup_test!($this.symmetric_difference_with($that));

        for bit in &block {
            assert!(!(lhs.contains(bit) && rhs.contains(bit)),
                    "{lhs:?} XOR {rhs:?}: block={block:?}",
                    lhs = lhs, rhs = rhs, block=block);
        }
        let expect = {
            use pairwise::symmetric_difference;
            let pair = symmetric_difference(lhs.iter(), rhs.iter());
            pair.collect::<Vec16>().count_ones()
        };
        assert!(block.count_ones() == expect,
                "{lhs:?} XOR {rhs:?}: got={got:?} want={want:?}",
                lhs = lhs, rhs = rhs, got = block.count_ones(), want = expect);
    };
}
