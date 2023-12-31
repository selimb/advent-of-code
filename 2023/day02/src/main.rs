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

struct Bag {
    blue: u64,
    red: u64,
    green: u64,
}

struct GameSet {
    blue: u64,
    red: u64,
    green: u64,
}

impl GameSet {
    pub fn parse(s: &str) -> Result<Self, String> {
        let mut ret = Self {
            blue: 0,
            red: 0,
            green: 0,
        };
        let roll_re = regex!(r"(\d+) (\S+)");
        let rolls = s.split(",").map(|s| s.trim());
        for roll in rolls {
            let caps = roll_re
                .captures(roll)
                .ok_or_else(|| format!("Invalid roll string: {roll}"))?;
            let num = caps[1].parse::<u64>().unwrap();
            match &caps[2] {
                "blue" => ret.blue = num,
                "red" => ret.red = num,
                "green" => ret.green = num,
                x => return Err(format!("invalid color '{}'", x)),
            }
        }
        Ok(ret)
    }

    pub fn check_possible(&self, bag: &Bag) -> Result<(), String> {
        let mut errors = Vec::new();
        for (name, actual, max) in [
            ("blue", self.blue, bag.blue),
            ("red", self.red, bag.red),
            ("green", self.green, bag.green),
        ] {
            if actual > max {
                errors.push(format!("expected at most {max} {name}, got {actual}"))
            }
        }

        if errors.is_empty() {
            return Ok(());
        } else {
            return Err(errors.join("; "));
        }
    }
}

struct Game {
    id: u64,
    sets: Vec<GameSet>,
}

impl Game {
    pub fn parse(line: &str) -> Result<Self, String> {
        let header_re = regex!(r"^Game (\d+):(.*)");
        let caps = header_re
            .captures(line)
            .ok_or_else(|| "line does not start with a game header")?;

        let game_id = caps[1]
            .parse::<u64>()
            .expect("Should be able to parse digits into a number.");

        let body = &caps[2];

        let mut sets: Vec<GameSet> = Vec::new();
        for s in body.split(";").map(|s| s.trim()) {
            let set = GameSet::parse(&s)?;
            sets.push(set);
        }
        Ok(Self { id: game_id, sets })
    }

    pub fn check_possible(&self, bag: &Bag) -> Result<(), String> {
        let mut impossible: Vec<String> = Vec::new();
        for (set_idx, set) in self.sets.iter().enumerate() {
            let setno = set_idx + 1;
            if let Err(err) = set.check_possible(bag) {
                impossible.push(format!("Set {setno} is impossible: {err}."));
            }
        }

        if impossible.is_empty() {
            return Ok(());
        } else {
            return Err(impossible.join(" "));
        }
    }

    fn compute_power(&self) -> u64 {
        let mut smallest_bag = Bag {
            blue: 0,
            green: 0,
            red: 0,
        };
        for set in self.sets.iter() {
            smallest_bag.blue = cmp::max(smallest_bag.blue, set.blue);
            smallest_bag.green = cmp::max(smallest_bag.green, set.green);
            smallest_bag.red = cmp::max(smallest_bag.red, set.red);
        }
        let power = smallest_bag.blue * smallest_bag.green * smallest_bag.red;
        power
    }
}

fn iter_games(path: &str) -> Result<impl Iterator<Item = Result<Game, String>>, String> {
    let file = fs::File::open(path).map_err(|err| err.to_string())?;
    let reader = std::io::BufReader::new(file);

    return Ok(reader
        .lines()
        .enumerate()
        .map(|(line_idx, line)| -> Result<Game, String> {
            let lineno = line_idx + 1;
            let line = line.map_err(|err| format!("Encountered error on line {lineno}. {err}"))?;

            let game = Game::parse(&line)
                .map_err(|err| format!("Failed parsing game on line {}: {line}. {err}", lineno))?;
            Ok(game)
        }));
}

fn run_1(path: &str) -> Result<(), String> {
    let bag = Bag {
        red: 12,
        green: 13,
        blue: 14,
    };
    let mut acc: u64 = 0;
    for game in iter_games(path)? {
        let game = game?;
        match game.check_possible(&bag) {
            Ok(_) => acc += game.id,
            Err(err) => eprintln!("Game {} is not possible. {}", game.id, err),
        }
    }
    println!("Answer: {acc}");

    Ok(())
}

fn run_2(path: &str) -> Result<(), String> {
    let mut acc: u64 = 0;
    for game in iter_games(path)? {
        let game = game?;
        let power = game.compute_power();
        acc += power;
    }
    println!("Answer: {acc}");

    Ok(())
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

    match args.part {
        PuzzlePart::One => run_1(&args.path),
        PuzzlePart::Two => run_2(&args.path),
    }
}
