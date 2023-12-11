use std::collections::HashSet;
use std::fs;
use std::io::BufRead;

use clap::{Parser, ValueEnum};

// https://github.com/matklad/once_cell/blob/master/examples/regex.rs
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

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

    let path = args.path;

    match args.part {
        PuzzlePart::One => run_1(&path),
        PuzzlePart::Two => run_2(&path),
    }
}

fn run_1(path: &String) -> Result<(), String> {
    let file = fs::File::open(&path).map_err(|err| err.to_string())?;
    let mut reader = std::io::BufReader::new(file);

    let mut acc = 0;
    for (idx, card) in iter_cards(&mut reader).enumerate() {
        let lineno = idx + 1;
        let card = card.map_err(|err| format!("Failed to parse card on line {lineno}: {err}"))?;
        let score = card.compute_score();
        acc += score;
    }
    println!("Answer: {acc}");
    Ok(())
}

fn run_2(_path: &String) -> Result<(), String> {
    todo!("")
}

struct Card {
    #[allow(dead_code)]
    id: usize,
    winning: HashSet<u16>,
    hand: HashSet<u16>,
}

impl Card {
    fn compute_score(&self) -> i64 {
        let match_count: usize = self.hand.intersection(&self.winning).count();
        let score = if match_count == 0 {
            0
        } else {
            2_i64.pow((match_count - 1).try_into().unwrap())
        };
        score
    }
}

fn iter_cards<R: BufRead>(reader: &mut R) -> impl Iterator<Item = Result<Card, String>> + '_ {
    fn parse_numbers(s: &str) -> HashSet<u16> {
        s.split_ascii_whitespace()
            .map(|chunk| chunk.trim().parse().unwrap())
            .collect()
    }

    let card_re = regex!(r"Card\s+(\d+):(.*)");
    reader
        .lines()
        .filter_map(|line| line.ok())
        .map(|line| -> Result<Card, String> {
            let line = line;
            let cap = card_re
                .captures(&line)
                .ok_or_else(|| "Regex did not match")?;

            let card_id: usize = cap[1]
                .parse()
                .expect("Should have been able to parse digits as number");

            let rest = &cap[2];
            let (winning, hand) = rest.split_once("|").ok_or("No | delimiter")?;

            let winning = parse_numbers(winning);
            let hand = parse_numbers(hand);
            let card = Card {
                id: card_id,
                winning,
                hand,
            };
            Ok(card)
        })
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test_Card {
    use super::*;

    #[test]
    fn test_compute_score() {
        let id = 1;
        let winning = HashSet::from([1, 2, 3, 4]);
        let cards = [
            // Zero winn
            Card {
                id,
                winning: winning.clone(),
                hand: [100, 101].into(),
            },
            Card {
                id,
                winning: winning.clone(),
                hand: [1, 100, 101].into(),
            },
            Card {
                id,
                winning: winning.clone(),
                hand: [1, 2, 999].into(),
            },
            Card {
                id,
                winning: winning.clone(),
                hand: [1, 2, 3, 4, 999, 777].into(),
            },
        ];

        let scores: Vec<_> = cards.iter().map(|c| c.compute_score()).collect();
        assert_eq!(scores, [0, 1, 2, 8]);
    }
}
