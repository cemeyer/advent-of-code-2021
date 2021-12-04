#![allow(dead_code, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use std::collections::*;

type Num = i16;

struct Board {
    board: Vec<Vec<Num>>,
    rows_unmarked: Vec<HashSet<Num>>,
    cols_unmarked: Vec<HashSet<Num>>,
    histo: HashMap<Num, u64>,
    sum_unmarked: u64,
    won: bool,
}

impl Board {
    fn new(board: Vec<Vec<Num>>) -> Self {
        let nrows = board.len();
        let ncols = board[0].len();

        let mut rows = Vec::new();
        let mut cols = Vec::new();
        let mut histo = HashMap::new();
        let mut total = 0;

        for (y, row) in board.iter().enumerate() {
            if rows.len() <= y {
                rows.push(HashSet::new());
            }
            for (x, val) in row.iter().enumerate() {
                if cols.len() <= x {
                    cols.push(HashSet::new());
                }

                cols[x].insert(*val);
                rows[y].insert(*val);

                let count = histo.get(val).copied().unwrap_or(0);
                histo.insert(*val, count + 1);

                total += *val as u64;
            }
        }

        Self {
            board,
            rows_unmarked: rows,
            cols_unmarked: cols,
            histo,
            sum_unmarked: total,
            won: false,
        }
    }

    fn mark(&mut self, mark: Num) -> bool {
        let count = if let Some(count) = self.histo.get(&mark) {
            count
        } else {
            return false;
        };

        self.sum_unmarked -= (mark as u64) * count;

        for i in 0..5 {
            self.rows_unmarked[i].remove(&mark);
            self.cols_unmarked[i].remove(&mark);
        }
        for i in 0..5 {
            if self.rows_unmarked[i].is_empty() {
                self.won = true;
                return true;
            }
            if self.cols_unmarked[i].is_empty() {
                self.won = true;
                return true;
            }
        }
        false
    }
}

struct Game {
    marks: Vec<Num>,
    boards: Vec<Board>,
}

impl Game {
    fn new() -> Self {
        Self { marks: Vec::new(), boards: Vec::new(), }
    }
}

fn parse(input: &str) -> Game {
    let mut result = Game::new();

    let mut lines = input.lines();

    let markline = lines.next().unwrap();
    for mark in markline.split(',') {
        result.marks.push(mark.parse::<Num>().unwrap());
    }

    // blank line
    while lines.next().is_some() {
        let mut board = Vec::new();
        for y in 0..5 {
            let mut row = Vec::new();
            let line = lines.next().unwrap();
            for val in line.split_ascii_whitespace() {
                row.push(val.parse::<Num>().unwrap());
            }
            assert_eq!(row.len(), 5);
            board.push(row);
        }
        assert_eq!(board.len(), 5);

        result.boards.push(Board::new(board));
    }

    result
}

fn part1(input: &str) -> String {
    let game = parse(input);
    let marks = game.marks;
    let mut boards = game.boards;

    for mark in marks.iter() {
        for board in boards.iter_mut() {
            if board.mark(*mark) {
                dbg!(mark, board.sum_unmarked);
                return format!("{}", (*mark as u64) * board.sum_unmarked);
            }
        }
    }
    unreachable!();
}

fn part2(input: &str) -> String {
    let game = parse(input);
    let marks = game.marks;
    let mut boards = game.boards;

    let mut last_win = 0;

    for mark in marks.iter() {
        for board in boards.iter_mut() {
            if board.won {
                continue;
            }
            if board.mark(*mark) {
                dbg!(mark, board.sum_unmarked);
                last_win = (*mark as u64) * board.sum_unmarked;
                dbg!(last_win);
            }
        }
    }
    format!("{}", last_win)
}

fn submit(puzzle: &mut aoc::Puzzle, part: aoc::Part, answ: &str) -> Result<()> {
    println!("Submitting: {} for part {:?}", answ, part);
    puzzle.submit_answer(part, answ)
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new2021(4)?;
    let data = puzzle.get_data()?;

    //let answ1 = part1(data);
    //submit(&mut puzzle, aoc::Part::One, &answ1)?;
    let answ2 = part2(data);
    submit(&mut puzzle, aoc::Part::Two, &answ2)?;

    Ok(())
}
