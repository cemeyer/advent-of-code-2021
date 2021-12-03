#![allow(dead_code, unused_macros)]

use anyhow::{anyhow, Context, Result};
use std::io::ErrorKind;

// Work around Rust's inability to concatenate / format const strings.
macro_rules! DAY_URI {
    () => { "https://adventofcode.com/{year}/day/{day}" };
}
macro_rules! INPUT_URI {
    () => { concat!(DAY_URI!(), "/input") };
}
macro_rules! SUBMIT_URI {
    () => { concat!(DAY_URI!(), "/answer") };
}
macro_rules! INPUT_PATH {
    () => { "day{day}.in" };
}
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.54 Safari/537.36";

fn get_session() -> Result<String> {
    std::fs::read_to_string("session.id")
        .map(|sid| sid.trim_end().to_owned())
        .with_context(|| "Reading AoC session cookie from \"session.id\"")
}

fn try_read_input(day: u16) -> Result<Option<String>> {
    match std::fs::read_to_string(format!(INPUT_PATH!(), day = day)) {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e)?,
    }
}

fn write_input(day: u16, input: &str) -> Result<()> {
    Ok(std::fs::write(format!(INPUT_PATH!(), day = day), input)?)
}

#[derive(Clone, Debug)]
pub struct Puzzle {
    year: u16,
    day: u16,
    session: String,
    input: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Part {
    One,
    Two,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SubmitError {
    AlreadyCompleted,
    // Brute-force prevention cooldown
    TooRecent(u32),
    // Wrong, too high
    TooHigh,
    // Wrong, too low
    TooLow,
    // Wrong, no reason mentioned
    Wrong(String),

    // Unexpected result
    Unexpected(String),
}

impl From<Part> for &str {
    fn from(part: Part) -> &'static str {
        match part {
            Part::One => "1",
            Part::Two => "2",
        }
    }
}

impl std::fmt::Display for SubmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl Puzzle {
    pub fn new(year: u16, day: u16, session: String, input: Option<String>) -> Self {
        Self { year, day, session, input, }
    }

    pub fn new2021(day: u16) -> Result<Self> {
        Ok(Self::new(2021, day, get_session()?, try_read_input(day)?))
    }

    pub fn get_data(&mut self) -> Result<&str> {
        if self.input.is_none() {
            self.fetch_data()?;
        }
        Ok(self.input.as_ref().unwrap())
    }

    fn apply_common_cookies(&self, req: ureq::Request) -> ureq::Request {
        req
            .set("Cookie", &format!("session={}", self.session))
            .set("Referer", &format!(DAY_URI!(), year = self.year, day = self.day))
            .set("User-Agent", USER_AGENT)
    }

    fn fetch_data(&mut self) -> Result<()> {
        // HTTP fetch
        let uri = format!(INPUT_URI!(), day = self.day, year = self.year);
        let resp = self.apply_common_cookies(ureq::get(&uri))
            .call()
            .with_context(|| format!("Fetching data for {:?}", self))?;

        let body = resp.into_string()?;

        // Cache
        write_input(self.day, &body)?;

        self.input = Some(body);
        Ok(())
    }

    // TODO: cache submitted outputs to avoid repeats
    pub fn submit_answer(&mut self, part: Part, answ: &str) -> Result<()> {
        if answ.len() == 0 {
            return Err(anyhow!("Refusing to submit empty answer"));
        }

        // HTTP POST
        let uri = format!(SUBMIT_URI!(), day = self.day, year = self.year);
        let resp = self.apply_common_cookies(ureq::post(&uri))
            .send_form(&[
                       ("level", part.into()),
                       ("answer", answ),
            ])?;

        let body = resp.into_string()?;
        std::fs::write(".submit.debug.body", &body).ok();

        if body.contains("That's the right answer") {
            if self.day == 25 && part == Part::One {
                // XXX only valid if we have the other 49 stars.
                // self.submit_answer(Part::Two, "done")?;
            }
            return Ok(());
        }

        if body.contains("Did you already complete it") {
            return Err(anyhow!(SubmitError::AlreadyCompleted));
        }
        if body.contains("You gave an answer too recently") {
            // TODO: parse "You have (?:(\d+)m )?(\d+)s left to wait"
            return Err(anyhow!(SubmitError::TooRecent(9999)));
        }

        if body.contains("That's not the right answer") {
            if body.contains("Your answer is too high") {
                return Err(anyhow!(SubmitError::TooHigh));
            }
            if body.contains("Your answer is too low") {
                return Err(anyhow!(SubmitError::TooLow));
            }
            return Err(anyhow!(SubmitError::Wrong(body)));
        }

        return Err(anyhow!(SubmitError::Unexpected(body)));
    }
}
