use std::{io, path::Path};

fn main() -> io::Result<()> {
    rrr_table("table.rs", 127)
}

fn rrr_table<P: AsRef<Path>>(path: P, n: usize) -> io::Result<()> {
    use std::{env, fs::File, io::Write};

    fn gentab(size: usize) -> Vec<Vec<u128>> {
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
    let mut file = File::create(Path::new(&dir).join(path))?;
    writeln!(file, "{:?}", gentab(n))
}
