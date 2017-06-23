macro_rules! delegate {
    ( $this: ident, $method: ident $(, $args: expr )* ) => {{
        match $this {
            Seq16(data) => data.$method( $( $args ),* ),
            Seq64(data) => data.$method( $( $args ),* ),
            Rle16(data) => data.$method( $( $args ),* ),
        }
    }};
    ( ref $this: ident, $method: ident $(, $args: expr )* ) => {{
        match *$this {
            Seq16(ref data) => data.$method( $( $args ),* ),
            Seq64(ref data) => data.$method( $( $args ),* ),
            Rle16(ref data) => data.$method( $( $args ),* ),
        }
    }};
    ( ref mut $this: ident, $method: ident $(, $args: expr )* ) => {{
        match *$this {
            Seq16(ref mut data) => data.$method( $( $args ),* ),
            Seq64(ref mut data) => data.$method( $( $args ),* ),
            Rle16(ref mut data) => data.$method( $( $args ),* ),
        }
    }}
}
