use crate::io::Write;
use std::{env, fs, io, path::Path};

fn rrr_table<P: AsRef<Path>>(path: P, arrty: &str, n: usize) -> io::Result<()> {
    fn gen_rrr_table(size: usize) -> Vec<Vec<u128>> {
        let mut table = vec![vec![0u128; size]; size];
        for k in 0..size {
            table[k][k] = 1; // initialize diagonal
            table[0][k] = 0; // initialize first row
            table[k][0] = 1; // initialize first col
        }
        for i in 1..size {
            for j in 1..size {
                table[i][j] = table[i - 1][j - 1] + table[i - 1][j];
            }
        }
        table
    }

    let dir = env::var("OUT_DIR").unwrap();
    let mut file = fs::File::create(Path::new(&dir).join(path))?;
    writeln!(
        file,
        r#"#[allow(clippy::unreadable_literal)]
pub static TABLE: {} = {:#?};
"#,
        arrty,
        gen_rrr_table(n)
    )
}

#[rustfmt::skip]
fn main() -> io::Result<()> {
    // rrr_table( "table15.rs",  "[[ u16;   15];  15]", 15)?;
    // rrr_table( "table31.rs",  "[[ u32;   31];  31]", 31)?;
    // rrr_table( "table63.rs",  "[[ u64;   63];  63]", 63)?;
    // rrr_table( "table127.rs", "[[u128;  127]; 127]", 127)?;

    rrr_table( "table.rs", "[[u128;  127]; 127]", 127)?;
    Ok(())
}
