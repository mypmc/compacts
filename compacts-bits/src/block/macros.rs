macro_rules! insert_all {
    ( $rle:expr $(, $x:expr )* ) => ($(
        assert!($rle.insert($x));
    )*)
}

macro_rules! remove_all {
    ( $rle:expr $(, $x:expr )* ) => ($(
        assert!($rle.remove($x));
    )*)
}
