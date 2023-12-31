use itertools::Itertools;
use std::collections::{HashMap, HashSet};
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

type Result<T> = core::result::Result<T, String>;

fn main() -> Result<()> {
    let args = Args::parse();

    let path = args.path;

    match args.part {
        PuzzlePart::One => run_1(&path),
        PuzzlePart::Two => run_2(&path),
    }
}

fn run_1(path: &str) -> Result<()> {
    let file = fs::File::open(path).map_err(|err| err.to_string())?;
    let mut reader = std::io::BufReader::new(file);

    let seeds = read_seeds(&mut reader)?;
    let mut src_curr = "seed".to_string();
    let mut seeds_mapped: HashMap<Id, Id> =
        HashMap::from_iter(seeds.iter().map(|seed_id| (*seed_id, *seed_id)));
    // NOTE: This is slightly more complicated than it should be, since we try to
    // simultaneously parse and process.
    loop {
        let map = match read_map(&mut reader, &src_curr) {
            Ok(Some(map)) => Ok(map),
            Ok(None) => break,
            Err(err) => Err(err),
        }?;
        for seed_dst in seeds_mapped.values_mut() {
            *seed_dst = map.lookup(*seed_dst);
        }
        src_curr = map.dst;
    }
    if src_curr != "location" {
        return Err(format!(
            "Expected last map destination to be 'location'. Got '{src_curr}'."
        ));
    }

    println!(
        "{}",
        seeds_mapped
            .iter()
            .map(|(seed_id, location_id)| format!("{seed_id} -> {location_id}"))
            .fold("".to_string(), |acc, s| acc + &s + "\n")
    );
    let lowest_location = seeds_mapped.values().min().ok_or("No location!")?;
    println!("Answer: {lowest_location}");

    Ok(())
}

fn run_2(path: &str) -> Result<()> {
    let file = fs::File::open(path).map_err(|err| err.to_string())?;
    let mut reader = std::io::BufReader::new(file);

    let seed_ranges = read_seed_ranges(&mut reader)?;
    let mut almanac: Vec<Map> = vec![];
    // NOTE: Can't parse+process as we did in part one, because there are too many damn seeds!
    //   So we instead parse the entire almanac, and then loop over seeds.
    loop {
        let map = match read_map(&mut reader, almanac.last().map_or("seed", |map| &map.dst)) {
            Ok(Some(map)) => Ok(map),
            Ok(None) => break,
            Err(err) => Err(err),
        }?;
        almanac.push(map);
    }
    match almanac.last() {
        Some(map) => {
            if map.dst != "location" {
                return Err(format!(
                    "Expected last map destination to be 'location'. Got '{}'.",
                    map.dst
                ));
            }
        }
        None => {
            return Err("Empty almanac!".to_string());
        }
    }

    let location_id = par_find_lowest_location(seed_ranges, almanac);
    println!("Answer: {location_id}");

    Ok(())
}

fn par_find_lowest_location(seed_ranges: Vec<SeedRange>, almanac: Vec<Map>) -> Id {
    // use rayon::prelude::*;
    use std::ops::Rem;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::Instant;

    // Logging stuff
    let start = Instant::now();
    let seed_count: Id = seed_ranges.iter().map(|r| r.len).sum();
    let counter = Arc::new(Mutex::new(0_i64));
    let log_every = ((seed_count as f64) / 100.0).ceil() as i64;

    let seed_ids = seed_ranges
        .iter()
        .flat_map(|seed_range| (seed_range.start..seed_range.start + seed_range.len));
    let best_location = seed_ids
        // TODO: This is slower with par_bridge o.O?
        // .par_bridge()
        .map(|seed_id| -> Id {
            let location_id = &almanac
                .iter()
                .fold(seed_id, |id_curr, map| map.lookup(id_curr));

            let counter_new = {
                let mut c = counter.lock().unwrap();
                *c = *c + 1;
                *c
            };
            if counter_new.rem(log_every) == 0 {
                let elapsed = (Instant::now() - start).as_secs_f32();
                let percent = (counter_new as f64) / (seed_count as f64) * 100_f64;
                println!("[{elapsed:.2}] {counter_new} / {seed_count} ({percent:.0} %)");
            }

            *location_id
        })
        .min()
        .unwrap();

    best_location
}

type Id = i64;

fn read_seeds<R: BufRead>(reader: &mut R) -> Result<HashSet<Id>> {
    let line = reader
        .lines()
        .next()
        .ok_or_else(|| "Unexpected EOF")?
        .map_err(|err| err.to_string())?;
    if !line.starts_with("seeds:") {
        return Err(format!(
            "Expected line to start with 'seeds: '. Got '{line}'."
        ));
    }
    let (_, seeds) = line.split_once(":").unwrap();

    let seeds: HashSet<Id> = parse_space_sep_numbers(seeds).collect();
    Ok(seeds)
}

struct SeedRange {
    start: Id,
    len: Id,
}

fn read_seed_ranges<R: BufRead>(reader: &mut R) -> Result<Vec<SeedRange>> {
    let line = reader
        .lines()
        .next()
        .ok_or_else(|| "Unexpected EOF")?
        .map_err(|err| err.to_string())?;
    if !line.starts_with("seeds:") {
        return Err(format!(
            "Expected line to start with 'seeds: '. Got '{line}'."
        ));
    }
    let (_, seeds) = line.split_once(":").unwrap();

    let seeds = parse_space_sep_numbers(seeds)
        .tuples()
        .map(|(start, len)| SeedRange { start, len })
        .collect();
    Ok(seeds)
}

fn read_map<R: BufRead>(reader: &mut R, src_expected: &str) -> Result<Option<Map>> {
    // Too lazy to propagate BufReader errors.
    let mut line_iter = reader.lines().map(|line| line.unwrap());

    let header = match (&mut line_iter.by_ref())
        .skip_while(|line| line.is_empty())
        .next()
    {
        Some(line) => line,
        None => return Ok(None),
    };
    let header_re = regex!(r"(\w+)-to-(\w+) map");
    let caps = match header_re.captures(&header) {
        Some(caps) => caps,
        None => return Err(format!("Invalid map header line: '{header}'")),
    };

    let src = caps[1].to_string();
    if src != src_expected {
        return Err(format!(
            "Expected map source to be '{src_expected}'. Got '{src}'."
        ));
    }
    let dst = caps[2].to_string();

    let ranges = line_iter
        .take_while(|line| !line.is_empty())
        .map(|line| {
            let numbers: Vec<_> = parse_space_sep_numbers(&line).collect();
            if numbers.len() != 3 {
                return Err(format!(
                    "Expected exactly 3 numbers. Got {}: {:?}",
                    numbers.len(),
                    numbers
                ));
            }
            Ok(MapRange {
                dst_start: numbers[0],
                src_start: numbers[1],
                len: numbers[2],
            })
        })
        .collect::<Result<Vec<MapRange>>>()?;

    Ok(Some(Map { src, dst, ranges }))
}

struct Map {
    #[allow(dead_code)]
    pub src: String,
    pub dst: String,
    pub ranges: Vec<MapRange>,
}

impl Map {
    fn lookup(&self, src_id: Id) -> Id {
        for range in &self.ranges {
            if src_id >= range.src_start {
                let delta = src_id - range.src_start;
                if delta < range.len {
                    return range.dst_start + delta;
                }
            }
        }
        return src_id;
    }
}

struct MapRange {
    dst_start: Id,
    src_start: Id,
    len: Id,
}

fn parse_space_sep_numbers<'a>(s: &'a str) -> impl Iterator<Item = Id> + 'a {
    s.trim()
        .split_ascii_whitespace()
        .map(|n| n.trim().parse::<Id>().expect(&format!("not a number: {n}")))
}

#[allow(non_snake_case)]
#[cfg(test)]
mod test_Map {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn test_lookup() {
        let txt = vec!["seed-to-soil map:", "50 98 2", "52 50 48"].join("\n");
        let mut reader = BufReader::new(txt.as_bytes());
        let map = read_map(&mut reader, "seed").unwrap().unwrap();

        let matrix: HashMap<Id, Id> = HashMap::from([
            // src, dst
            (0, 0),
            (1, 1),
            (48, 48),
            (50, 52),
            (51, 53),
            (96, 98),
            (97, 99),
            (98, 50),
            (99, 51),
        ]);
        let actual: HashMap<Id, Id> = HashMap::from_iter(
            matrix
                .keys()
                .map(|seed_id| (*seed_id, map.lookup(*seed_id))),
        );

        assert_eq!(actual, matrix);
    }
}
