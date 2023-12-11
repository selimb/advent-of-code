use std::{collections::HashMap, fs, io::BufRead};

use once_cell::sync::Lazy;

#[allow(dead_code)]
fn parse_calibration_values1(line: &str) -> Result<u8, String> {
    let mut first_digit: Option<char> = None;
    let mut last_digit: Option<char> = None;
    for c in line.chars() {
        if !c.is_ascii_digit() {
            continue;
        }
        if first_digit.is_none() {
            first_digit = Some(c);
        } else {
            last_digit = Some(c);
        }
    }

    let first_digit = first_digit.ok_or_else(|| "Expected at least 1 digit.")?;
    let last_digit = last_digit.unwrap_or(first_digit);

    let mut digits = first_digit.to_string();
    digits.push(last_digit);

    let value = digits
        .parse::<u8>()
        .expect("Should be able to parse two digits as an integer");
    Ok(value)
}

static DIGIT_STRINGS: Lazy<HashMap<&str, u8>> = Lazy::new(|| {
    HashMap::from([
        ("one", 1),
        ("two", 2),
        ("three", 3),
        ("four", 4),
        ("five", 5),
        ("six", 6),
        ("seven", 7),
        ("eight", 8),
        ("nine", 9),
    ])
});

fn parse_digit(line: &str, c: char, idx: usize) -> Option<u8> {
    let slice = &line[idx..];
    if let Ok(digit) = c.to_string().parse::<u8>() {
        return Some(digit);
    }
    // TODO: Probably doesn't handle Unicode very well?
    for (&pattern, &digit) in DIGIT_STRINGS.iter() {
        if slice.starts_with(pattern) {
            return Some(digit);
        }
    }
    return None;
}

fn parse_calibration_values2(line: &str) -> Result<u8, String> {
    let mut first_digit: Option<u8> = None;
    let mut last_digit: Option<u8> = None;
    for (idx, c) in line.char_indices() {
        if let Some(digit) = parse_digit(line, c, idx) {
            if first_digit.is_none() {
                first_digit = Some(digit);
            } else {
                last_digit = Some(digit);
            }
        }
    }

    let first_digit = first_digit.ok_or_else(|| "Expected at least 1 digit.")?;
    let last_digit = last_digit.unwrap_or(first_digit);

    let value = format!("{first_digit}{last_digit}")
        .parse::<u8>()
        .expect("Should be able to parse two digits as an integer");
    Ok(value)
}

fn main() -> Result<(), String> {
    let args = std::env::args().collect::<Vec<_>>();
    let fpath = args
        .get(1)
        .ok_or_else(|| "Expected at least one argument")?;

    let file = fs::File::open(fpath).map_err(|err| format!("{err}"))?;

    let mut acc: u64 = 0;
    let reader = std::io::BufReader::new(file);
    for (lineno, line) in reader.lines().enumerate() {
        let line = line.map_err(|err| format!("Encountered error on line {lineno}: {err}"))?;

        let value = parse_calibration_values2(&line)
            .map_err(|err| format!("Encountered error on line {lineno}: {err}"))?;
        acc += value as u64;
    }
    println!("{acc}");
    Ok(())
}
