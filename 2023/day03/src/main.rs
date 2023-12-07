use std::io::BufRead;
use std::{cmp, fs};

use clap::{Parser, ValueEnum};

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

    for (y, line_str) in reader.lines().enumerate() {
        let line_str = line_str.map_err(|err| err.to_string())?;
        let line = parse_line(&line_str, y);
    }
    // =============
    // NEW CODE HERE
    // =============

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct Number {
    value: u64,
    start_x: usize,
    end_x: usize,
    y: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct Symbol {
    value: char,
    x: usize,
    y: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct Line {
    numbers: Vec<Number>,
    symbols: Vec<Symbol>,
}

fn parse_line(line: &str, y: usize) -> Line {
    let mut numbers: Vec<Number> = Vec::new();
    let mut symbols: Vec<Symbol> = Vec::new();

    let mut number_string: Option<String> = None;
    let mut number_start: usize = 0;
    let mut number_end: usize = 0;

    for (idx, c) in line.char_indices() {
        if c.is_ascii_digit() {
            match number_string {
                Some(ref mut string_curr) => {
                    // Accumulate to the current number
                    string_curr.push(c);
                    number_end = idx;
                }
                None => {
                    // Start a new number
                    number_string = Some(c.to_string());
                    number_start = idx;
                    number_end = idx;
                }
            };
        } else {
            // Maybe flush the current number
            if let Some(ref string_curr) = number_string.take() {
                let value = string_curr.parse::<u64>().unwrap();
                numbers.push(Number {
                    value,
                    start_x: number_start,
                    end_x: number_end,
                    y,
                });
            }
            // Maybe add symbol
            if c != '.' {
                symbols.push(Symbol {
                    value: c,
                    x: idx,
                    y,
                });
            }
        }
    }
    Line { numbers, symbols }
}

#[cfg(test)]
mod test_parse_line {
    use super::*;

    #[test]
    fn it_works() {
        let y = 42;
        let line_str = "467..*114$..#";
        let line = parse_line(line_str, y);

        assert_eq!(
            line.numbers,
            vec![
                Number {
                    value: 467,
                    start_x: 0,
                    end_x: 2,
                    y
                },
                Number {
                    value: 114,
                    start_x: 6,
                    end_x: 8,
                    y
                }
            ]
        );
        assert_eq!(
            line.symbols,
            vec![
                Symbol {
                    value: '*',
                    x: 5,
                    y,
                },
                Symbol {
                    value: '$',
                    x: 9,
                    y,
                },
                Symbol {
                    value: '#',
                    x: 12,
                    y,
                },
            ]
        );
    }
}
