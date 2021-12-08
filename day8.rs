#![allow(dead_code, unused_assignments, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use graphlib::{Graph, VertexId};
use ndarray::prelude::*;
use ndarray::{ArcArray2, parallel::par_azip};
use std::cmp::{min, max};
use std::convert::{TryFrom, TryInto};
use std::collections::*;
use std::hash::Hash;
use std::iter::FromIterator;

fn part1(inputs: &Vec<(Vec<&str>, Vec<&str>)>) -> i64 {
    let mut t1478 = 0;
    for row in inputs {
        let (_, outputs) = row;
        for output in outputs {
            if output.len() == 2 || output.len() == 4 ||
                output.len() == 3 || output.len() == 7 {
                    t1478 += 1;
                }
        }
    }

    t1478
}

#[derive(Hash, Debug, PartialEq, Ord, PartialOrd, Eq, Clone, Copy)]
enum Seg {
    A,B,C,D,E,F,G
}

fn si(s: Seg) -> usize {
    s as usize
}

fn segchar(s: Seg) -> char {
    match s {
        Seg::A => 'a',
        Seg::B => 'b',
        Seg::C => 'c',
        Seg::D => 'd',
        Seg::E => 'e',
        Seg::F => 'f',
        Seg::G => 'g',
    }
}

fn charseg(c: char) -> Seg {
    match c {
        'a' => Seg::A,
        'b' => Seg::B,
        'c' => Seg::C,
        'd' => Seg::D,
        'e' => Seg::E,
        'f' => Seg::F,
        'g' => Seg::G,
        _ => { unreachable!(); }
    }
}

fn part2(inputs: &Vec<(Vec<&str>, Vec<&str>)>) -> u64 {
    // segs -> digit mapping
    let mut segdigit = HashMap::new();
    segdigit.insert(
        BTreeSet::from_iter([Seg::A, Seg::B, Seg::C, Seg::E, Seg::F, Seg::G].iter().cloned()),
        '0');
    segdigit.insert(
        BTreeSet::from_iter([Seg::C, Seg::F].iter().cloned()),
        '1');
    segdigit.insert(
        BTreeSet::from_iter([Seg::A, Seg::C, Seg::D, Seg::E, Seg::G].iter().cloned()),
        '2');
    segdigit.insert(
        BTreeSet::from_iter([Seg::A, Seg::C, Seg::D, Seg::F, Seg::G].iter().cloned()),
        '3');
    segdigit.insert(
        BTreeSet::from_iter([Seg::B, Seg::C, Seg::D, Seg::F].iter().cloned()),
        '4');
    segdigit.insert(
        BTreeSet::from_iter([Seg::A, Seg::B, Seg::D, Seg::F, Seg::G].iter().cloned()),
        '5');
    segdigit.insert(
        BTreeSet::from_iter([Seg::A, Seg::B, Seg::D, Seg::E, Seg::F, Seg::G].iter().cloned()),
        '6');
    segdigit.insert(
        BTreeSet::from_iter([Seg::A, Seg::C, Seg::F].iter().cloned()),
        '7');
    segdigit.insert(
        BTreeSet::from_iter([Seg::A, Seg::B, Seg::C, Seg::D, Seg::E, Seg::F, Seg::G].iter().cloned()),
        '8');
    segdigit.insert(
        BTreeSet::from_iter([Seg::A, Seg::B, Seg::C, Seg::D, Seg::F, Seg::G].iter().cloned()),
        '9');

    //  aaaa
    // b    c
    // b    c
    //  dddd
    // e    f
    // e    f
    //  gggg

    let mut total = 0;
    for row in inputs {
        let (signals, outputs) = row;
        // XXX test input
        //let signals = &"acedgfb cdfbe gcdfa fbcad dab cefabd cdfgeb eafb cagedb ab".split(" ").collect::<Vec<_>>();
        //let outputs = &"cdfeb fcadb cdfeb cdbaf".split(" ").collect::<Vec<_>>();

        // Observed Seg -> Set<Actual poss. Seg>
        let mut candidates = HashMap::new();
        for i in [Seg::A, Seg::B, Seg::C, Seg::D, Seg::E, Seg::F, Seg::G].iter() {
            candidates.insert(i, HashSet::new());
            for j in "abcdefg".chars() {
                candidates.get_mut(&i).unwrap().insert(j);
            }
        }

        // First, process 1, 4, and 7.  8 gives no diagnostic information.
        for signal in signals.iter().chain(outputs) {
            match signal.len() {
                // "1"
                2 => {
                    candidates.get_mut(&Seg::C).unwrap().retain(|seg| signal.contains(*seg));
                    candidates.get_mut(&Seg::F).unwrap().retain(|seg| signal.contains(*seg));
                }
                // "4"
                4 => {
                    candidates.get_mut(&Seg::B).unwrap().retain(|seg| signal.contains(*seg));
                    candidates.get_mut(&Seg::C).unwrap().retain(|seg| signal.contains(*seg));
                    candidates.get_mut(&Seg::D).unwrap().retain(|seg| signal.contains(*seg));
                    candidates.get_mut(&Seg::F).unwrap().retain(|seg| signal.contains(*seg));
                }
                // "7"
                3 => {
                    candidates.get_mut(&Seg::A).unwrap().retain(|seg| signal.contains(*seg));
                    candidates.get_mut(&Seg::C).unwrap().retain(|seg| signal.contains(*seg));
                    candidates.get_mut(&Seg::F).unwrap().retain(|seg| signal.contains(*seg));
                }
                // "2", "5", "3"
                5 => {
                    // all share A/D/G segments
                    candidates.get_mut(&Seg::A).unwrap().retain(|seg| signal.contains(*seg));
                    candidates.get_mut(&Seg::D).unwrap().retain(|seg| signal.contains(*seg));
                    candidates.get_mut(&Seg::G).unwrap().retain(|seg| signal.contains(*seg));
                }
                // "6", "9", "0"
                6 => {
                    candidates.get_mut(&Seg::A).unwrap().retain(|seg| signal.contains(*seg));
                    candidates.get_mut(&Seg::B).unwrap().retain(|seg| signal.contains(*seg));
                    candidates.get_mut(&Seg::F).unwrap().retain(|seg| signal.contains(*seg));
                    candidates.get_mut(&Seg::G).unwrap().retain(|seg| signal.contains(*seg));
                }
                _ => {}
            }
        }
        dbg!(&candidates);

        let mut any = true;
        while any {
            any = false;
            let mut remove = Vec::new();
            for (seg, val) in candidates.iter() {
                if val.len() == 1 {
                    remove.push(*val.iter().next().unwrap());
                }
            }
            for (&seg, val) in candidates.iter_mut() {
                for x in remove.iter() {
                    if val.len() != 1 && val.remove(&x) {
                        dbg!(seg, &val);
                        any = true;
                    }
                }
            }
            if any {
                dbg!(&candidates);
            }
        }

        let mut mapping = HashMap::new();
        for (&seg, vals) in candidates.iter() {
            assert_eq!(vals.len(), 1);
            //mapping.insert(segchar(*seg), *vals.iter().next().unwrap()); // backwards!
            mapping.insert(*vals.iter().next().unwrap(), segchar(*seg));
        }
        dbg!(&mapping);

        let digits = outputs.iter().map(|out| {
            let corrected = out.chars().map(|c| charseg(mapping[&c])).collect::<BTreeSet<_>>();
            //dbg!(out, &corrected);
            segdigit[&corrected]
        });

        let num = String::from_iter(digits);
        dbg!(&num);
        let num = num.parse::<u64>().unwrap();

        total += num;
    }

    total
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 8)?;
    let data = puzzle.get_data()?;
    let lines = data.lines().collect::<Vec<_>>();
    let inputs = lines.iter().map(|line| {
        let (left, right) = line.split_once(" | ").unwrap();
        let signals = left.split(' ').collect::<Vec<_>>();
        let outputs = right.split(' ').collect::<Vec<_>>();
        (signals, outputs)
    }).collect::<Vec<_>>();

    //let answ1 = part1(&inputs);
    //dbg!(&answ1);
    //puzzle.submit_answer(aoc::Part::One, &format!("{}", answ1))?;

    let answ2 = part2(&inputs);
    dbg!(&answ2);
    //puzzle.submit_answer(aoc::Part::Two, &format!("{}", answ2))?;

    Ok(())
}
