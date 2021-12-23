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

#[derive(Eq,PartialEq,Clone,Debug,Hash,PartialOrd,Ord)]
struct GameState {
    /// Location (row, column) of each Amphipod, indexed by type (A = 0, B = 1, ...) and member of
    /// that type (0..=1).
    locs: [[(u8, u8); 2]; 4],
}

/// Win condition - all amphipods in their correct rooms.
#[inline]
fn win(state: &GameState) -> bool {
    let locs = state.locs;
    if locs[0][0].1 != 3 ||
        locs[0][1].1 != 3 ||
        locs[1][0].1 != 5 ||
        locs[1][1].1 != 5 ||
        locs[2][0].1 != 7 ||
        locs[2][1].1 != 7 ||
        locs[3][0].1 != 9 ||
        locs[3][1].1 != 9 {
            return false;
        }
    for l in 0..4 {
        for i in 0..2 {
            if locs[l][i].0 < 2 ||
                locs[l][i].0 > 3 {
                    return false;
                }
        }
    }
    true
}

/// COST to move corresponding Amphipod one square.
const COST: [isize;4] = [1, 10, 100, 1000];

/// Generate new state reflecting moving 'mover' to 'dest' on the existing board 'state'.
///
/// Checks that the path through the hallway is clear using 'hallway_occ'.
///
/// Returns `None` if the hallway is obstructed, or `Some((cost, newstate))` if the move was
/// possible.
#[inline]
fn gen_state(state: &GameState, hallway_occ: &[bool;12], mover: (usize,usize), dest: (u8,u8)) -> Option<(i64, GameState)> {
    let start = state.locs[mover.0][mover.1];
    let dx = if start.1 < dest.1 { 1isize } else { -1 };

    // Check that the walk is unobstructed.
    let mut x = (start.1 as isize + dx);
    while x != dest.1 as isize {
        if hallway_occ[x as usize] {
            return None;
        }
        x += dx;
    }
    // Need to check the x == dest.1 case
    if hallway_occ[x as usize] {
        return None;
    }

    // Manhattan distance.
    let xdist = ((start.1 as isize) - (dest.1 as isize)).abs();
    let ydist = ((start.0 as isize) - (dest.0 as isize)).abs();

    let mut newstate = state.clone();
    newstate.locs[mover.0][mover.1] = dest;

    let cost = COST[mover.0] * (xdist + ydist);
    Some((cost as i64, newstate))
}

fn part1() -> i64 {
    // I skipped parsing and just hand-entered the starting coordinates so that my game state could
    // be pretty small.
    let mut locs = [[(0,0);2];4];

    // 'A' locations
    locs[0][0] = (3,5);
    locs[0][1] = (2,7);

    // 'B' locations, etc
    locs[1][0] = (2,5);
    locs[1][1] = (3,9);

    locs[2][0] = (3,3);
    locs[2][1] = (2,9);

    locs[3][0] = (2,3);
    locs[3][1] = (3,7);

    // SAMPLE - override locations with those of the Part 1 sample.
    //locs[0][0] = (3,3);
    //locs[0][1] = (3,9);
    //locs[1][0] = (2,3);
    //locs[1][1] = (2,7);
    //locs[2][0] = (2,5);
    //locs[2][1] = (3,7);
    //locs[3][0] = (3,5);
    //locs[3][1] = (2,9);

    // Basic game-tree search using best-first search on the cost function, which we need to
    // minimize.
    let mut queue = BinaryHeap::new();
    let mut visited = HashSet::new();
    let start = GameState { locs, };
    queue.push((0i64, start.clone()));
    visited.insert(start);

    while queue.len() != 0 {
        let (sco, state) = queue.pop().unwrap();
        //dbg2!(-sco);
        //dbg2!(&state);
        if win(&state) {
            // Cost is negative because we want to minimize, but Rust's BinaryHeap is a max-heap.
            return -sco;
        }

        // Generate some expanded state to quickly answer valid-move questions.
        let mut room_occ = [0u8; 4]; // how many correct species are in a room, or is it wrong-type occupied?
        let mut hallway_occ = [false; 12]; // hallway coord index occupied
        let mut room_top_occ = [false; 10]; // room[2] coord index occupied
        for species in 0..4 {
            for indv in 0..2 {
                let coords = state.locs[species][indv];
                if coords.0 >= 2 {
                    if coords.0 == 2 {
                        room_top_occ[coords.1 as usize] = true;
                    }
                    let room_idx = (coords.1 - 3) as usize/2;
                    if (species*2 + 3) != coords.1 as usize {
                        room_occ[room_idx] = 0x80;
                    } else {
                        if room_occ[room_idx] != 0x80 {
                            room_occ[room_idx] += 1;
                        }
                    }
                } else {
                    hallway_occ[coords.1 as usize] = true;
                }
            }
        }
        //dbg2!(&room_occ);
        //dbg2!(&hallway_occ);

        // Generate valid moves.
        for species in 0..4 {
            // Destination x of this species' organized room.
            let dest_room_x = (species*2 + 3) as u8;
            for indv in 0..2 {
                // we can move (1) from starting room to any hallway square, except a doorway;
                // (1)(a) also inside starting room
                // (2) from hallway to final room, iff devoid of other pods
                // that's it?  cannot move between two hallway squares
                let coords = state.locs[species][indv];

                // Not strictly necessary, but avoid exploring some useless state.
                if coords.1 == dest_room_x {
                    // Won't move if in final room, most remote square
                    if coords.0 == 3 {
                        continue;
                    }
                    // Or near-square, but the other occupant is correct.
                    if room_occ[species] == 2 {
                        continue;
                    }
                }

                // Starting in hallway; we can (only) move to the destination room.
                if coords.0 == 1 {
                    let occ = room_occ[species];
                    // Can't move in if a different species is present.
                    if occ == 0x80 {
                        continue;
                    }
                    assert!(occ == 0 || occ == 1);
                    // Always move into the furthest unoccupied space (optimal).
                    let dest_y = 3 - occ;
                    if let Some((cost, my_move)) = gen_state(&state, &hallway_occ, (species, indv), (dest_y, dest_room_x)) {
                        if visited.contains(&my_move) {
                            continue;
                        }
                        //dbg2!((cost, &my_move));
                        visited.insert(my_move.clone());
                        queue.push((sco - cost, my_move));
                    }
                } else {
                    // Starting in a room; can (only) go to the hallway.
                    let dest_y = 1;

                    // Can't move to 3,5,7,9 (doorways).  Hallway spans 1..=11.
                    for dest_x in [1,2,4,6,8,10,11] {
                        // Can't go through another amphipod.
                        if coords.0 == 3 && room_top_occ[coords.1 as usize] {
                            continue;
                        }
                        if let Some((cost, my_move)) = gen_state(&state, &hallway_occ, (species, indv), (dest_y, dest_x)) {
                            if visited.contains(&my_move) {
                                continue;
                            }
                            //dbg2!((cost, &my_move));
                            visited.insert(my_move.clone());
                            queue.push((sco - cost, my_move));
                        }
                    }
                }
            }
        }
    }

    unreachable!();
}

//////////////////////////////////////////////////////////////////////////////////
// PART 2
//////////////////////////////////////////////////////////////////////////////////

#[derive(Eq,PartialEq,Clone,Debug,Hash,PartialOrd,Ord)]
struct GameState2 {
    /// Location (row, column) of each Amphipod, indexed by type (A = 0, B = 1, ...) and member of
    /// that type (0..=3).
    locs: [[(u8, u8); 4]; 4],
}

/// Win condition - all amphipods in their correct rooms.
#[inline]
fn win2(state: &GameState2) -> bool {
    let locs = state.locs;
    if locs[0][0].1 != 3 ||
        locs[0][1].1 != 3 ||
        locs[0][2].1 != 3 ||
        locs[0][3].1 != 3 ||
        locs[1][0].1 != 5 ||
        locs[1][1].1 != 5 ||
        locs[1][2].1 != 5 ||
        locs[1][3].1 != 5 ||
        locs[2][0].1 != 7 ||
        locs[2][1].1 != 7 ||
        locs[2][2].1 != 7 ||
        locs[2][3].1 != 7 ||
        locs[3][0].1 != 9 ||
        locs[3][1].1 != 9 ||
        locs[3][2].1 != 9 ||
        locs[3][3].1 != 9 {
            return false;
        }
    for l in 0..4 {
        for i in 0..4 {
            if locs[l][i].0 < 2 ||
                locs[l][i].0 > 5 {
                    return false;
                }
        }
    }
    true
}

/// Generate new state reflecting moving 'mover' to 'dest' on the existing board 'state'.
///
/// Checks that the path through the hallway is clear using 'hallway_occ'.
///
/// Returns `None` if the hallway is obstructed, or `Some((cost, newstate))` if the move was
/// possible.
#[inline]
fn gen_state2(state: &GameState2, hallway_occ: &[bool;12], mover: (usize,usize), dest: (u8,u8)) -> Option<(i64, GameState2)> {
    let start = state.locs[mover.0][mover.1];
    let dx = if start.1 < dest.1 { 1isize } else { -1 };

    // Check that the walk is unobstructed.
    let mut x = (start.1 as isize + dx);
    while x != dest.1 as isize {
        if hallway_occ[x as usize] {
            return None;
        }
        x += dx;
    }
    // Need to check the x == dest.1 case
    if hallway_occ[x as usize] {
        return None;
    }

    // Manhattan distance.
    let xdist = ((start.1 as isize) - (dest.1 as isize)).abs();
    let ydist = ((start.0 as isize) - (dest.0 as isize)).abs();

    let mut newstate = state.clone();
    newstate.locs[mover.0][mover.1] = dest;

    let cost = COST[mover.0] * (xdist + ydist);
    Some((cost as i64, newstate))
}

fn part2() -> i64 {
    // As in part 1, I just skipped parsing and hand-entered the starting coordinates.
    let mut locs = [[(0,0);4];4];

    // 'A' locations
    locs[0][0] = (5,5);
    locs[0][1] = (2,7);

    // 'B' locations, etc
    locs[1][0] = (2,5);
    locs[1][1] = (5,9);

    locs[2][0] = (5,3);
    locs[2][1] = (2,9);

    locs[3][0] = (2,3);
    locs[3][1] = (5,7);

    // 3 #D#C#B#A#
    // 4 #D#B#A#C#
    locs[0][2] = (4,7);
    locs[0][3] = (3,9);

    locs[1][2] = (4,5);
    locs[1][3] = (3,7);

    locs[2][2] = (3,5);
    locs[2][3] = (4,9);

    locs[3][2] = (3,3);
    locs[3][3] = (4,3);

    // SAMPLE - override locations with those of the Part 1 sample.
    //locs[0][0] = (5,3);
    //locs[0][1] = (5,9);
    //locs[1][0] = (2,3);
    //locs[1][1] = (2,7);
    //locs[2][0] = (2,5);
    //locs[2][1] = (5,7);
    //locs[3][0] = (5,5);
    //locs[3][1] = (2,9);

    // Basic game-tree search using best-first search on the cost function, which we need to
    // minimize.
    let mut queue = BinaryHeap::new();
    let mut visited = HashSet::new();
    let start = GameState2 { locs, };
    queue.push((0i64, start.clone()));
    visited.insert(start);

    let mut _n = 0;
    while queue.len() != 0 {
        let (sco, state) = queue.pop().unwrap();
        //dbg2!(-sco);
        //dbg2!(&state);
        if win2(&state) {
            // Cost is negative because we want to minimize, but Rust's BinaryHeap is a max-heap.
            return -sco;
        }

        // Generate some expanded state to quickly answer valid-move questions.
        let mut room_occ = [0u8; 4]; // how many correct species are in a room, or is it wrong-type occupied?
        let mut hallway_occ = [false; 12]; // hallway coord index occupied
        let mut room_top_occ = [[false;3]; 10]; // room[2..=4] coord index occupied
        for species in 0..4 {
            for indv in 0..4 {
                let coords = state.locs[species][indv];
                if coords.0 >= 2 {
                    if coords.0 < 5 {
                        room_top_occ[coords.1 as usize][coords.0 as usize - 2] = true;
                    }
                    let room_idx = (coords.1 - 3) as usize/2;
                    if (species*2 + 3) != coords.1 as usize {
                        room_occ[room_idx] = 0x80;
                    } else {
                        if room_occ[room_idx] != 0x80 {
                            room_occ[room_idx] += 1;
                        }
                    }
                } else {
                    hallway_occ[coords.1 as usize] = true;
                }
            }
        }
        //dbg2!(&room_occ);
        //dbg2!(&hallway_occ);
        //dbg2!(&room_top_occ);

        // Generate valid moves.
        for species in 0..4 {
            // Destination x of this species' organized room.
            let dest_room_x = (species*2 + 3) as u8;
            for indv in 0..4 {
                // we can move (1) from starting room to any hallway square, except a doorway;
                // (1)(a) also inside starting room, if non-final
                // (2) from hallway to final room, iff devoid of other pods
                // that's it?  cannot move between two hallway squares

                let coords = state.locs[species][indv];

                // Not strictly necessary, but avoid exploring some useless state.
                if coords.1 == dest_room_x {
                    // Won't move if in final room, most remote square, or similarly packed in.
                    if room_occ[species] != 0x80 && room_occ[species] + coords.0 >= 6 {
                        continue;
                    }
                }

                // Starting in hallway; we can (only) move to the destination room.
                if coords.0 == 1 {
                    let occ = room_occ[species];
                    // Can't move in if a different species is present.
                    if occ == 0x80 {
                        continue;
                    }
                    // Always move into the furthest unoccupied space (optimal).
                    let dest_y = 5 - occ;
                    if let Some((cost, my_move)) = gen_state2(&state, &hallway_occ, (species, indv), (dest_y, dest_room_x)) {
                        if visited.contains(&my_move) {
                            continue;
                        }
                        //dbg2!("hall2room", (cost, &my_move));
                        visited.insert(my_move.clone());
                        queue.push((sco - cost, my_move));
                    }
                } else {
                    // Starting in a room; can (only) go to the hallway.
                    let dest_y = 1;

                    // Can't move to 3,5,7,9 (doorways).  Hallway spans 1..=11.
                    for dest_x in [1,2,4,6,8,10,11] {
                        // Can't go through another amphipod.
                        if coords.0 > 2 && room_top_occ[coords.1 as usize][0..(coords.0 as usize - 2)].iter().any(|o| *o) {
                            continue;
                        }
                        if let Some((cost, my_move)) = gen_state2(&state, &hallway_occ, (species, indv), (dest_y, dest_x)) {
                            if visited.contains(&my_move) {
                                continue;
                            }
                            //dbg2!("room2hall", (cost, &my_move));
                            visited.insert(my_move.clone());
                            queue.push((sco - cost, my_move));
                        }
                    }
                }
            }
        }
    }

    unreachable!();
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 23)?;
    let _data = puzzle.get_data()?;

    let answ1 = part1();
    dbg!(&answ1);
    //assert_ne!(answ1, 15282);
    //assert!(answ1 > 15332); // "Too low"
    assert_eq!(answ1, 15538);
    let answ2 = part2();
    dbg!(&answ2);
    assert_eq!(answ2, 47258);
    Ok(())
}
