use std::fs;
use std::io::BufRead;

use clap::Parser;

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

    fn check_possible(&self, bag: &Bag) -> Result<(), String> {
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
}

#[derive(Parser, Debug)]
#[command()]
struct Args {
    path: String,
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    let file = fs::File::open(args.path).map_err(|err| err.to_string())?;
    let reader = std::io::BufReader::new(file);

    let bag = Bag {
        red: 12,
        green: 13,
        blue: 14,
    };
    let mut acc: u64 = 0;
    for (line_idx, line) in reader.lines().enumerate() {
        let lineno = line_idx + 1;
        let line = line.map_err(|err| format!("Encountered error on line {lineno}. {err}"))?;

        let game = Game::parse(&line)
            .map_err(|err| format!("Failed parsing game on line {}: {line}. {err}", lineno))?;

        match game.check_possible(&bag) {
            Ok(_) => acc += game.id,
            Err(err) => eprintln!("Game {} is not possible. {}", game.id, err),
        }
    }
    println!("Answer: {acc}");

    Ok(())
}
