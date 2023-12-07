use std::io::BufRead;
use std::{cmp, fs};

use clap::{Parser, ValueEnum};

// https://github.com/matklad/once_cell/blob/master/examples/regex.rs
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

// #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
// enum PuzzlePart {
//     One,
//     Two,
// }

#[derive(Parser, Debug)]
#[command()]
struct Args {
    path: String,
    // #[arg(short, long, value_enum)]
    // part: PuzzlePart,
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    let path = args.path;
    let file = fs::File::open(&path).map_err(|err| err.to_string())?;
    let reader = std::io::BufReader::new(file);

    // =============
    // NEW CODE HERE
    // =============

    Ok(())
}
