#![allow(dead_code)]

use anyhow::{anyhow, Result};
use std::io::ErrorKind;

// Work around Rust's inability to concatenate const strings.
macro_rules! DAY_URI {
    () => { "https://adventofcode.com/{year}/day/{day}" };
}
const INPUT_URI: &str = concat!(DAY_URI!(), "/input");
const SUBMIT_URI: &str = concat!(DAY_URI!(), "/answer");
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.54 Safari/537.36";

fn get_session() -> Result<String> {
    // e.g., open("session.id").read().strip()
    Ok("XXX".to_owned())
}

fn try_read_input(day: u16) -> Result<Option<String>> {
    match std::fs::read_to_string(format!("day{day}.in", day = day)) {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e)?,
    }
}

pub struct Puzzle {
    year: u16,
    day: u16,
    session: String,
    input: Option<String>,
    // TODO: cache submitted outputs to avoid repeats
}

impl Puzzle {
    pub fn new(year: u16, day: u16, session: String, input: Option<String>) -> Self {
        Self { year, day, session, input, }
    }

    pub fn new2021(day: u16) -> Result<Self> {
        Ok(Self::new(2021, day, get_session()?, try_read_input(day)?))
    }

    pub fn get_data(&mut self) -> Result<&str> {
        if let Some(x) = &self.input {
            return Ok(&x);
        }
        // TODO: HTTP fetch
        return Err(anyhow!("blah"));
    }

    // TODO: submit
}
