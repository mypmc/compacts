#[macro_export]
macro_rules! bitset {
    ( $( $bit:expr ),* ) => {
        {
            let mut bits = $crate::bits::Set::new();
            $( bits.put( $bit, true ); )*
            bits.optimize();
            bits
        }
    }
}

macro_rules! divrem {
    ( $bit:expr, $n:expr ) => {
        {
            let q = $bit / $n;
            let r = $bit % $n;
            (q as usize, r)
        }
    }
}

/// Find the smallest index i in range at which f(i) is true, assuming that
/// f(i) == true implies f(i+1) == true.
macro_rules! search {
    ( $start:expr, $end:expr, $func:expr ) => {
        {
            let mut i = $start;
            let mut j = $end;
            while i < j {
                let h = i + (j - i) / 2;
                if $func(h) {
                    j = h; // f(j) == true
                } else {
                    i = h + 1; // f(i-1) == false
                }
            }
            i
        }
    }
}

macro_rules! select_by_rank {
    ( 0, $this:ident, $count:expr, $start:expr, $end:expr, $out:ty ) => {
        {
            let fun = |i| $this.rank0(i as $out) > $count;
            let pos = search!($start, $end, fun);
            if pos < $end {
                Some(pos as $out - 1)
            } else {
                None
            }
        }
    };
    ( 1, $this:ident, $count:expr, $start:expr, $end:expr, $out:ty ) => {
        {
            let fun = |i| $this.rank1(i as $out) > $count;
            let pos = search!($start, $end, fun);
            if pos < $end {
                Some(pos as $out - 1)
            } else {
                None
            }
        }
    }
}
