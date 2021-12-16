#![allow(dead_code, unused_assignments, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use bitvec::prelude::*;
use graphlib::{Graph, VertexId};
use itertools::iproduct;
use ndarray::prelude::*;
use ndarray::{ArcArray2, parallel::par_azip};
use std::cmp::{min, max};
use std::convert::{TryFrom, TryInto};
use std::collections::*;
use std::hash::Hash;
use std::iter::FromIterator;

use aoc::{dbg2, byte, ByteString};

#[derive(Eq,PartialEq,Clone,Debug,Hash)]
struct EvalResult {
    value: u64,
    sum_versions: u64,
    parsed_length: usize, // in bits
}

/// Evaluate the Packet tree represented by this sequence of bits, recursing to evaluate nested
/// packets.
fn eval(input: &BitSlice::<Msb0, u8>) -> EvalResult {
    // The first three bits encode the packet _version_.
    let version = input[0..3].load_be::<u8>();
    // The next three bits encode the packet _type ID_.
    let type_ = input[3..6].load_be::<u8>();

    // Track total parsed length, in bits, so that recursive callers know how far forward to skip.
    // XXX should instead use a consuming iterator, but whatever.
    let mut pktlen = 6;
    let mut sumversions = version as u64;

    let mut cursor = &input[6..];
    let pktval = match type_ {
        // Packets with type ID 4 represent a _literal value_.
        0x4 => {
            let mut res = 0u64;
            // Literals are represented using a UTF8-like encoding; each nibble is prefixed with a
            // continue bit.
            loop {
                let next = cursor[0..5].load_be::<u8>();
                cursor = &cursor[5..];
                pktlen += 5;

                // The nibbles are MSB.
                res = res << 4;
                res |= (next & 0xf) as u64;

                // Check the continue bit.
                if (next & 0x10) == 0 {
                    break;
                }
            }

            // Evaluated result of a literal value is just the value.
            res
        }
        // Every other type of packet represents an _operator_.
        _ => {
            // Shared parsing logic for operator packets: first bit determines _length type ID_.
            let lentype = cursor[0];
            cursor = &cursor[1..];
            pktlen += 1;

            // Store the evaluated value of each sub-Packet (sub-expression).
            let mut subpackets = Vec::new();

            match lentype {
                // If the length type ID is 0, then the next 15 bits represent the _total length in
                // bits_ of the contained sub-packets.
                false => {
                    let tlen = cursor[0..15].load_be::<u16>();
                    cursor = &cursor[15..];
                    pktlen += 15;

                    // Evaluate enough packets, recursively, to consume the appropriate number of
                    // bits.
                    let mut sublen = 0;
                    while sublen < tlen as usize {
                        let eresult = eval(cursor);
                        subpackets.push(eresult.value);
                        sumversions += eresult.sum_versions;
                        let len = eresult.parsed_length;
                        sublen += len;
                        pktlen += len;
                        cursor = &cursor[len..];
                    }
                    assert_eq!(sublen, tlen as usize);
                }
                // If the length type ID is 1, then the next 11 bits represent the _number of
                // sub-packets immediately contained_ by this packet.
                true => {
                    let nsubpackets = cursor[0..11].load_be::<u16>();
                    cursor = &cursor[11..];
                    pktlen += 11;

                    // Evaluate the appropriate number of subpackets.
                    for _i in 0..nsubpackets {
                        let eresult = eval(cursor);
                        subpackets.push(eresult.value);
                        sumversions += eresult.sum_versions;
                        let len = eresult.parsed_length;
                        cursor = &cursor[len..];
                        pktlen += len;
                    }
                }
            }

            // Re-dispatch on type ID, for the operator subset of types (part 2).
            match type_ {
                // Packets with type ID 0 are _sum_ packets.
                0 => subpackets.iter().sum::<u64>(),
                // Packets with type ID 1 are _product_ packets.
                1 => subpackets.iter().product(),
                // Packets with type ID 2 are _min_ packets.
                2 => *subpackets.iter().min().unwrap(),
                // Packets with type ID 3 are _max_ packets.
                3 => *subpackets.iter().max().unwrap(),
                // Packets with type ID 5 are _greater than_ packets.
                5 => if subpackets[0] > subpackets[1] { 1 } else { 0 },
                // Packets with type ID 6 are _less than_ packets.
                6 => if subpackets[0] < subpackets[1] { 1 } else { 0 },
                // Packets with type ID 7 are _equal to_ packets.
                7 => if subpackets[0] == subpackets[1] { 1 } else { 0 },
                _ => { unreachable!(); }
            }

        }
    };

    EvalResult {
        value: pktval,
        sum_versions: sumversions,
        parsed_length: pktlen,
    }
}

// What do you get if you evaluate the expression represented by your hexadecimal-encoded BITS
// transmission?
fn part2(input: &ParseResult) -> u64 {
    let bits = input.view_bits::<Msb0>();
    let EvalResult { value, .. } = eval(bits);
    value
}

// Parse the hierarchy of the packets throughout the transmission and _add up all of the version
// numbers_.
fn part1(input: &ParseResult) -> u64 {
    let bits = input.view_bits::<Msb0>();
    let EvalResult { sum_versions, .. } = eval(bits);
    sum_versions
}

type ParseResult = Vec<u8>;

// Yeah, this isn't really the parser, but usually input parsing is a smaller part of the puzzle.
fn parse(data: &str) -> ParseResult {
    hex::decode(data.trim_end()).unwrap()
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 16)?;
    let data = puzzle.get_data()?;
    //let data = "D2FE28";
    //let data = "38006F45291200";
    //let data = "EE00D40C823060";
    let parsed = parse(data);

    let answ1 = part1(&parsed);
    dbg!(&answ1);
    assert_eq!(answ1, 984);
    let answ2 = part2(&parsed);
    dbg!(&answ2);
    assert_eq!(answ2, 1015320896946);

    Ok(())
}
