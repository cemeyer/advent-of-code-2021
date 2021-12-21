#![allow(dead_code, unused_assignments, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use bitvec::prelude::*;
use graphlib::{Graph, VertexId};
use itertools::iproduct;
//use nalgebra::*;
use ndarray::prelude::*;
use ndarray::{ArcArray2, parallel::par_azip};
use std::cmp::{min, max};
use std::convert::{TryFrom, TryInto};
use std::collections::*;
use std::hash::Hash;
use std::iter::FromIterator;

use aoc::{dbg2, byte, BitCursor, ByteString};

type ParseResult = (usize, usize);

fn parse(data: &str) -> ParseResult {
    (7, 9) // Real
    //(4,8) // Sample
}

fn part1(input: &ParseResult) -> usize {
    let mut determin_die = (1..=100).cycle();

    let mut pos = vec![input.0, input.1];
    let mut scores = vec![0, 0];
    let mut player = 0;
    let mut nrolls = 0;

    loop {
        let roll = (0..3).map(|_| determin_die.next().unwrap()).sum::<usize>();
        nrolls += 3;
        pos[player] = ((pos[player] - 1 + roll) % 10) + 1;
        scores[player] += pos[player];

        if scores[player] > 1000 {
            break;
        }

        player = (player + 1) % 2;
    }


    nrolls * scores[(player + 1) % 2]
}

#[derive(Eq,PartialEq,Clone,Debug,Hash)]
struct State {
    pos: Vec<u8>,
    scores: Vec<u8>,
    player: u8,
    count: u64,
}

fn part2(input: &ParseResult) -> u64 {
    // Precompute counts of dice rolls.
    let rolls = {
        let mut histo = HashMap::new();
        for i in 1..=3 {
            for j in 1..=3 {
                for k in 1..=3 {
                    *histo.entry((i+j+k)).or_default() += 1;
                }
            }
        }
        let mut rolls = histo.iter().map(|(k,v)| (*k, *v)).collect::<Vec<(usize, usize)>>();
        rolls.sort();
        rolls
    };
    dbg2!(&rolls);

    // I expressed this as a game-tree search problem.
    //
    // We express the number of universes on the same "path" via the 'count' parameter.  Without
    // this, the number grows too quickly.  E.g., when you roll 3 dice, there is one universe where
    // you get 1+1+1, but there are 3 universes where you get 1+1+2.
    let state = State { pos: vec![input.0 as _, input.1 as _], scores: vec![0, 0], player: 0, count: 1 };

    // Since we need to evaluate the entire space, depth-first search is most efficient.
    let mut queue = Vec::new();
    queue.push(state);

    let mut wins = vec![0,0];

    while queue.len() > 0 {
        let next = queue.pop().unwrap();

        // Enumerate eligible moves.
        for (roll, newcounts) in rolls.iter() {
            let mut newstate = next.clone();
            newstate.pos[next.player as usize] = ((newstate.pos[next.player as usize] - 1 + *roll as u8) % 10) + 1;
            newstate.scores[next.player as usize] += newstate.pos[next.player as usize];
            newstate.count *= *newcounts as u64;

            // Count win conditions.
            if newstate.scores[next.player as usize] >= 21 {
                wins[next.player as usize] += newstate.count;
                continue;
            }

            newstate.player = (newstate.player + 1) % 2;
            queue.push(newstate);
        }
    }
    *wins.iter().max().unwrap()
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 21)?;
    let data = puzzle.get_data()?;
    let parsed = parse(data);

    let answ1 = part1(&parsed);
    dbg!(&answ1);
    assert_eq!(answ1, 679329);
    let answ2 = part2(&parsed);
    dbg!(&answ2);
    assert_eq!(answ2, 433315766324816);

    Ok(())
}
