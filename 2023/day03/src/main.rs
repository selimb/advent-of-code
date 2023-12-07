use std::collections::HashSet;
use std::fs;
use std::io::BufRead;

use clap::{Parser, ValueEnum};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum PuzzlePart {
    One,
    Two,
}

#[derive(Parser, Debug)]
#[command()]
struct Args {
    path: String,

    #[arg(short, long, value_enum)]
    part: PuzzlePart,
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    match args.part {
        PuzzlePart::One => run_1(&args.path),
        PuzzlePart::Two => run_2(&args.path),
    }
}

fn run_1(path: &String) -> Result<(), String> {
    let file = fs::File::open(&path).map_err(|err| err.to_string())?;
    let mut reader = std::io::BufReader::new(file);

    let grid = parse_grid(&mut reader);

    let mut acc: u64 = 0;
    // NOTE: Turns out this isn't necessary, but oh well.
    let mut numbers_seen: HashSet<NumberId> = HashSet::new();
    for (y, row) in grid.rows.iter().enumerate() {
        // xxx
        for symbol in &row.symbols {
            for number in find_adjancent_to_symbol(&symbol, y, &grid) {
                let first_seen = numbers_seen.insert(number.id);
                if first_seen {
                    acc += number.value;
                }
            }
        }
    }
    println!("Answer: {acc}");
    Ok(())
}

fn run_2(path: &String) -> Result<(), String> {
    println!("{path}");
    Ok(())
}

type NumberId = usize;

struct IdGenerator {
    curr: NumberId,
}

impl IdGenerator {
    pub fn new() -> Self {
        IdGenerator { curr: 0 }
    }

    pub fn next(&mut self) -> NumberId {
        let ret = self.curr;
        self.curr += 1;
        ret
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Number {
    id: NumberId,
    value: u64,
    start_x: usize,
    end_x: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct Symbol {
    value: char,
    x: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct Line {
    numbers: Vec<Number>,
    symbols: Vec<Symbol>,
}

fn parse_line(line: &str, id_gen: &mut IdGenerator) -> Line {
    let mut numbers: Vec<Number> = Vec::new();
    let mut symbols: Vec<Symbol> = Vec::new();

    let mut number_string: Option<String> = None;
    let mut number_start: usize = 0;
    let mut number_end: usize = 0;

    // DISCUSS: GAH, hard to write small helper closures that need to mutate things.
    // Captures mutable references to `numbers` and `id_gen`.
    let mut maybe_flush_number =
        |number_string: &mut Option<String>, number_start: usize, number_end: usize| -> () {
            // Maybe flush the current number
            if let Some(ref string_curr) = number_string.take() {
                let value = string_curr.parse::<u64>().unwrap();
                numbers.push(Number {
                    id: id_gen.next(),
                    value,
                    start_x: number_start,
                    end_x: number_end,
                });
            }
        };

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
            maybe_flush_number(&mut number_string, number_start, number_end);
            // Maybe add symbol
            if c != '.' {
                symbols.push(Symbol { value: c, x: idx });
            }
        }
    }
    // Also need to flush numbers that run until the end of the line
    maybe_flush_number(&mut number_string, number_start, number_end);
    Line { numbers, symbols }
}

struct Grid {
    rows: Vec<Line>,
}

fn parse_grid<R: BufRead>(reader: &mut R) -> Grid {
    let mut id_gen = IdGenerator::new();
    let mut rows: Vec<Line> = Vec::new();
    for line_str in reader.lines() {
        let line_str = line_str.unwrap();
        let line = parse_line(&line_str, &mut id_gen);
        rows.push(line);
    }
    Grid { rows }
}

/// Finds numbers on `grid` adjacent to `symbol` on line `y`.
fn find_adjancent_to_symbol<'a>(symbol: &Symbol, y: usize, grid: &'a Grid) -> Vec<&'a Number> {
    /// Assuming `symbol` and `number` are on adjacent rows, returns whether they're actually adjacent,
    /// i.e. adjacent in x.
    fn is_adgacent(symbol: &Symbol, number: &Number) -> bool {
        let xmin = if number.start_x == 0 {
            0
        } else {
            number.start_x - 1
        };
        let xmax = number.end_x + 1;

        let adjacent = (symbol.x >= xmin) && (symbol.x <= xmax);
        adjacent
    }

    let mut adjacent_rows = vec![y];
    if y > 0 {
        adjacent_rows.push(y - 1);
    }
    if y < grid.rows.len() - 1 {
        adjacent_rows.push(y + 1);
    }

    let mut matches: Vec<&'a Number> = Vec::new();
    for row_idx in adjacent_rows.iter() {
        let numbers_on_row = &grid.rows[*row_idx].numbers;
        for number in numbers_on_row {
            if is_adgacent(symbol, &number) {
                matches.push(&number);
            }
        }
    }
    matches
}

#[cfg(test)]
mod test_parse_line {
    use super::*;

    #[test]
    fn it_works() {
        let mut id_gen = IdGenerator::new();
        let line_str = "467..*114$..#9";
        let line = parse_line(line_str, &mut id_gen);

        assert_eq!(
            line.numbers,
            vec![
                Number {
                    id: 0,
                    value: 467,
                    start_x: 0,
                    end_x: 2,
                },
                Number {
                    id: 1,
                    value: 114,
                    start_x: 6,
                    end_x: 8,
                },
                Number {
                    id: 2,
                    value: 9,
                    start_x: 13,
                    end_x: 13,
                }
            ]
        );
        assert_eq!(
            line.symbols,
            vec![
                Symbol { value: '*', x: 5 },
                Symbol { value: '$', x: 9 },
                Symbol { value: '#', x: 12 },
            ]
        );
    }
}
