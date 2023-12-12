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

    let path = args.path;

    match args.part {
        PuzzlePart::One => run_1(&path),
        PuzzlePart::Two => run_2(&path),
    }
}

fn run_1(path: &str) -> Result<(), String> {
    let file = fs::File::open(&path).map_err(|err| err.to_string())?;
    let reader = std::io::BufReader::new(file);
    let mut lines = reader.lines();

    let times = parse_line1(&lines.next().unwrap().unwrap());
    let records = parse_line1(&lines.next().unwrap().unwrap());
    let races = times.iter().zip(records.iter()).map(|(time, record)| Race {
        time: *time,
        record: *record,
    });

    let mut acc = 1;
    for race in races {
        let better = calc_beat_record(&race);
        acc *= better.len();
    }

    println!("Answer: {acc}");
    Ok(())
}

fn run_2(path: &str) -> Result<(), String> {
    let file = fs::File::open(&path).map_err(|err| err.to_string())?;
    let reader = std::io::BufReader::new(file);
    let mut lines = reader.lines();

    let time = parse_line2(&lines.next().unwrap().unwrap());
    let record = parse_line2(&lines.next().unwrap().unwrap());
    let race = Race { time, record };

    let ret = calc_beat_record(&race).len();

    println!("Answer: {ret}");
    Ok(())
}

fn parse_line1(line: &str) -> Vec<i64> {
    parse_space_sep_numbers(line.split_once(":").unwrap().1.trim()).collect()
}

fn parse_line2(line: &str) -> i64 {
    line.split_once(":")
        .unwrap()
        .1
        .replace(" ", "")
        .parse()
        .unwrap()
}

fn parse_space_sep_numbers<'a>(s: &'a str) -> impl Iterator<Item = i64> + 'a {
    s.trim().split_ascii_whitespace().map(|n| {
        n.trim()
            .parse::<i64>()
            .expect(&format!("not a number: {n}"))
    })
}

struct Race {
    pub time: i64,
    pub record: i64,
}

fn calc_beat_record(race: &Race) -> Vec<i64> {
    let mut ret = Vec::new();
    for button_hold_time in 1..race.time {
        let velocity = button_hold_time;
        let time_left = race.time - button_hold_time;
        let distance = velocity * time_left;
        if distance > race.record {
            ret.push(button_hold_time);
        }
    }
    ret
}
