#![allow(dead_code, unused_macros)]

use anyhow::{anyhow, Context, Result};
use std::io::ErrorKind;

// Work around Rust's inability to concatenate / format const strings.
macro_rules! YEAR_URI {
    () => { "https://adventofcode.com/{year}" };
}
macro_rules! STATS_URI {
    () => { concat!(YEAR_URI!(), "/leaderboard/self") };
}
macro_rules! DAY_URI {
    () => { concat!(YEAR_URI!(), "/day/{day}") };
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

fn apply_common_cookies(req: ureq::Request, session: &str, year: u16) -> ureq::Request {
    req
        .set("Cookie", &format!("session={}", session))
        // For the common logic, use the year (calendar) page as Referer
        .set("Referer", &format!(YEAR_URI!(), year = year))
        .set("User-Agent", USER_AGENT)
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
        // For Puzzles, set Referer to that day's page
        apply_common_cookies(req, &self.session, self.year)
            .set("Referer", &format!(DAY_URI!(), year = self.year, day = self.day))
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
                let stats = get_stats_session(&self.session, self.year)
                    .with_context(||
                                  format!("Fetching stats to try autocompleting #50"))?;
                // Only valid if we have the other 49 stars.
                if stats.count_stars() == 49 {
                    return self.submit_answer(Part::Two, "done");
                }
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

#[derive(Clone, Debug)]
pub struct DayPartStat {
    pub time_hour: u8,
    pub time_min: u8,
    pub time_sec: u8,
    pub rank: u32,
    pub score: u8,
}

impl DayPartStat {
    fn new() -> Self {
        Self {
            time_hour: 0,
            time_min: 0,
            time_sec: 0,
            rank: 0,
            score: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DayStat {
    pub day: u8,
    pub part1: DayPartStat,
    pub part2: Option<DayPartStat>,
}

impl DayStat {
    fn new(day: u8) -> Self {
        Self { day, part1: DayPartStat::new(), part2: None, }
    }
}

#[derive(Clone, Debug)]
pub struct Stats {
    days: Vec<DayStat>,
}

impl Stats {
    fn new() -> Self {
        Self { days: Vec::new(), }
    }

    pub fn day(&self, n: usize) -> Option<&DayStat> {
        if n < 1 || n > self.days.len() {
            return None;
        }
        Some(&self.days[n - 1])
    }

    pub fn days(&self) -> usize {
        self.days.len()
    }

    pub fn count_stars(&self) -> usize {
        let mut res = 0;

        for day in self.days.iter() {
            res += 1;
            if day.part2.is_some() {
                res += 1;
            }
        }

        res
    }
}

pub fn get_stats(year: u16) -> Result<Stats> {
    let session = get_session()?;
    get_stats_session(&session, year)
}

fn get_stats_session(session: &str, year: u16) -> Result<Stats> {
    // HTTP fetch
    let uri = format!(STATS_URI!(), year = year);
    let resp = apply_common_cookies(ureq::get(&uri), session, year)
        .call()
        .with_context(|| format!("Fetching stats for {}", year))?;
    let body = resp.into_string()?;

    // World's worst HTML parser
    let mut pre_count = 0;
    let lines = body
        .lines()
        .filter(|line| {
            // We're capturing the lines between the <pre> tags, and we only expect one section.
            if line.contains("<pre") {
                assert_eq!(pre_count, 0);
                pre_count += 1;
                false
            } else if line.contains("</pre>") {
                assert_eq!(pre_count, 1);
                pre_count += 1;
                false
            } else {
                pre_count == 1
            }
        })
        .collect::<Vec<_>>();

    let mut res = Stats::new();
    for line in lines.iter().rev() {
        let mut words = line.split_ascii_whitespace();
        let word1 = words.next().unwrap();
        if word1 == "Day" {
            break;
        }
        let dayn = word1.parse::<u8>().unwrap();

        let mut day = DayStat::new(dayn);
        let mut dayparts = (0..2).map(|_| {
            let time = words.next().unwrap();
            if time == "-" {
                None
            } else {
                let mut time_parts = time.split(':');
                let time_hour = time_parts.next().unwrap().parse::<u8>().unwrap();
                let time_min = time_parts.next().unwrap().parse::<u8>().unwrap();
                let time_sec = time_parts.next().unwrap().parse::<u8>().unwrap();

                let rank = words.next().unwrap().parse::<u32>().unwrap();
                let score = words.next().unwrap().parse::<u8>().unwrap();

                let res = DayPartStat { time_hour, time_min, time_sec, rank, score, };
                Some(res)
            }
        });
        day.part1 = dayparts.next().unwrap().unwrap();
        day.part2 = dayparts.next().unwrap();

        assert_eq!(res.days.len() + 1, dayn as _);
        res.days.push(day);
    }

    Ok(res)
}
