#[macro_export]
macro_rules! bitmap {
    ( $( $bit:expr ),* ) => {
        {
            let mut bits = $crate::bits::Map::new();
            $(
                bits.insert( $bit );
            )*
            bits
        }
    }
}

macro_rules! keypos {
    ( $bit:expr, $key:ident, $pos:ident ) => (
        let key  = $bit / 64;
        let $pos = $bit % 64;
        let $key = key as usize;
    );
}

macro_rules! bitmask {
    ( $bit:expr, $key:ident, $mask:ident ) => (
        keypos!($bit, $key, pos);
        let $mask = 1 << pos;
    );
}

macro_rules! extend_by_u16 {
    ( $inserter: expr, $iter: expr ) => {
        {
            let mut weight = 0;
            for item in $iter {
                if $inserter.insert(item) {
                    weight += 1;
                }
            }
            weight
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
