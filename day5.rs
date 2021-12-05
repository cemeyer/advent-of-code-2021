#![allow(dead_code, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use std::collections::*;

type Num = i16;
type Pair = (Num, Num);

fn parse_pair(input: &str) -> Pair {
    let mut nums = input.split(',');
    let v1 = nums.next().unwrap().parse::<Num>().unwrap();
    let v2 = nums.next().unwrap().parse::<Num>().unwrap();
    (v1, v2)
}

fn parse(input: &str) -> Vec<(Pair, Pair)> {
    let mut res = Vec::new();
    for line in input.lines() {
        let mut pairs = line.split(" -> ");
        let p1 = parse_pair(pairs.next().unwrap());
        let p2 = parse_pair(pairs.next().unwrap());
        res.push((p1, p2));
    }
    res
}

fn fillx(lnpts: &mut HashSet<Pair>, ovlp: &mut HashSet<Pair>, x: Num, y1: Num, y2: Num) {
    if y1 > y2 {
        fillx(lnpts, ovlp, x, y2, y1);
        return;
    }
    for y in y1..=y2 {
        let pt = (x, y);
        if lnpts.contains(&pt) {
            ovlp.insert(pt);
        } else {
            lnpts.insert(pt);
        }
    }
}

fn filly(lnpts: &mut HashSet<Pair>, ovlp: &mut HashSet<Pair>, y: Num, x1: Num, x2: Num) {
    if x1 > x2 {
        filly(lnpts, ovlp, y, x2, x1);
        return;
    }
    for x in x1..=x2 {
        let pt = (x, y);
        if lnpts.contains(&pt) {
            ovlp.insert(pt);
        } else {
            lnpts.insert(pt);
        }
    }
}

fn part1(input: &str) -> String {
    let inp = parse(input);

    let mut lnpts = HashSet::new();
    let mut ovlp = HashSet::new();
    for line in inp.iter() {
        let ((x1, y1), (x2, y2)) = *line;

        if x1 == x2 {
            fillx(&mut lnpts, &mut ovlp, x1, y1, y2);
        } else if y1 == y2 {
            filly(&mut lnpts, &mut ovlp, y1, x1, x2);
        } else {
            continue;
        }
    }

    format!("{}", ovlp.len())
}

fn filldiag(lnpts: &mut HashSet<Pair>, ovlp: &mut HashSet<Pair>, pt1: Pair, pt2: Pair) {
    let (x1, y1) = pt1;
    let (x2, y2) = pt2;
    if x1 > x2 {
        filldiag(lnpts, ovlp, pt2, pt1);
        return;
    }

    let run = x2 - x1;
    let rise = y2 - y1;
    let dy = rise / run;
    assert!(dy == 1 || dy == -1);

    let mut y = y1;
    for x in x1..=x2 {
        let pt = (x, y);
        if lnpts.contains(&pt) {
            ovlp.insert(pt);
        } else {
            lnpts.insert(pt);
        }
        y += dy;
    }
}

fn part2(input: &str) -> String {
    let inp = parse(input);

    let mut lnpts = HashSet::new();
    let mut ovlp = HashSet::new();
    for line in inp.iter() {
        let ((x1, y1), (x2, y2)) = *line;

        if x1 == x2 {
            fillx(&mut lnpts, &mut ovlp, x1, y1, y2);
        } else if y1 == y2 {
            filly(&mut lnpts, &mut ovlp, y1, x1, x2);
        } else {
            filldiag(&mut lnpts, &mut ovlp, (x1, y1), (x2, y2));
        }
    }

    format!("{}", ovlp.len())
}

fn submit(puzzle: &mut aoc::Puzzle, part: aoc::Part, answ: &str) -> Result<()> {
    println!("Submitting: {} for part {:?}", answ, part);
    puzzle.submit_answer(part, answ)
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new2021(5)?;
    let data = puzzle.get_data()?;

    //let answ1 = part1(data);
    //submit(&mut puzzle, aoc::Part::One, &answ1)?;
    let answ2 = part2(data);
    submit(&mut puzzle, aoc::Part::Two, &answ2)?;

    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        let answ = super::part1(
"0,9 -> 5,9
8,0 -> 0,8
9,4 -> 3,4
2,2 -> 2,1
7,0 -> 7,4
6,4 -> 2,0
0,9 -> 2,9
3,4 -> 1,4
0,0 -> 8,8
5,5 -> 8,2");
        assert_eq!(answ, "5");
    }
}
