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

/// Cursor type to support cleaner parsing.
struct BitCursor<'a, E: BitOrder> {
    input: &'a BitSlice<E, u8>,
}

impl<'a, E: BitOrder> BitCursor<'a, E> {
    pub fn new(input: &'a BitSlice<E, u8>) -> Self {
        Self {
            input,
        }
    }

    /// Get the underlying bitslice at the current parse position.
    pub fn as_slice(&self) -> &'a BitSlice<E, u8> {
        self.input
    }
}

impl<'a> BitCursor<'a, Msb0> {
    /// Parse the first `bits` bits from this iterator, consuming them.  `T` should be as wide or
    /// wider than `bits`, probably.
    #[inline]
    pub fn parse_be<T: bitvec::mem::BitMemory>(&mut self, bits: usize) -> T {
        let res = self.peek_be::<T>(bits);
        self.input = &self.input[bits..];
        res
    }

    /// Parse the first `bits` bits from this iterator.  `T` should be as wide or wider than
    /// `bits`, probably.
    #[inline]
    fn peek_be<T: bitvec::mem::BitMemory>(&self, bits: usize) -> T {
        self.input[..bits].load_be::<T>()
    }
}

#[derive(Eq,PartialEq,Clone,Debug,Hash)]
struct EvalResult {
    value: u64,
    sum_versions: u64,
}

/// Evaluate the Packet tree represented by this sequence of bits, recursing to evaluate nested
/// packets.
fn eval(input: &mut BitCursor<Msb0>) -> EvalResult {
    // The first three bits encode the packet _version_.
    let version = input.parse_be::<u8>(3);
    // The next three bits encode the packet _type ID_.
    let type_ = input.parse_be::<u8>(3);

    let mut sumversions = version as u64;

    let pktval = match type_ {
        // Packets with type ID 4 represent a _literal value_.
        0x4 => {
            let mut res = 0u64;
            // Literals are represented using a UTF8-like encoding; each nibble is prefixed with a
            // continue bit.
            loop {
                let next = input.parse_be::<u8>(5);

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
            let lentype = input.parse_be::<u8>(1);

            // Store the evaluated value of each sub-Packet (sub-expression).
            let mut subpackets = Vec::new();

            match lentype {
                // If the length type ID is 0, then the next 15 bits represent the _total length in
                // bits_ of the contained sub-packets.
                0 => {
                    let tlen = input.parse_be::<u16>(15);
                    let future_pos = input.as_slice()[(tlen as usize)..].as_bitptr();

                    // Evaluate enough packets, recursively, to consume the appropriate number of
                    // bits.
                    while input.as_slice().as_bitptr() != future_pos {
                        let eresult = eval(input);
                        subpackets.push(eresult.value);
                        sumversions += eresult.sum_versions;
                    }
                }
                // If the length type ID is 1, then the next 11 bits represent the _number of
                // sub-packets immediately contained_ by this packet.
                1 => {
                    let nsubpackets = input.parse_be::<u16>(11);

                    // Evaluate the appropriate number of subpackets.
                    for _i in 0..nsubpackets {
                        let eresult = eval(input);
                        subpackets.push(eresult.value);
                        sumversions += eresult.sum_versions;
                    }
                }
                _ => { unreachable!(); }
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
    }
}

// What do you get if you evaluate the expression represented by your hexadecimal-encoded BITS
// transmission?
fn part2(input: &ParseResult) -> u64 {
    let bits = input.view_bits::<Msb0>();
    let mut curs = BitCursor::new(&bits);
    let EvalResult { value, .. } = eval(&mut curs);
    value
}

// Parse the hierarchy of the packets throughout the transmission and _add up all of the version
// numbers_.
fn part1(input: &ParseResult) -> u64 {
    let bits = input.view_bits::<Msb0>();
    let mut curs = BitCursor::new(&bits);
    let EvalResult { sum_versions, .. } = eval(&mut curs);
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
    let parsed = parse(data);

    let answ1 = part1(&parsed);
    dbg!(&answ1);
    assert_eq!(answ1, 984);
    let answ2 = part2(&parsed);
    dbg!(&answ2);
    assert_eq!(answ2, 1015320896946);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_eval(inp: &str) -> EvalResult {
        let parsed = parse(inp);
        let bits = parsed.view_bits::<Msb0>();
        let mut curs = BitCursor::new(&bits);
        eval(&mut curs)
    }

    #[test]
    fn test_p1() {
        let data = "D2FE28";
        let eres = test_eval(data);
        assert_eq!(eres.value, 2021);
        assert_eq!(eres.sum_versions, 6);

        let data = "38006F45291200";
        let eres = test_eval(data);
        assert_eq!(eres.value, 1); // (10 < 20)
        assert_eq!(eres.sum_versions, 9);

        let data = "EE00D40C823060";
        let eres = test_eval(data);
        assert_eq!(eres.value, 3); // max(1,2,3)
        assert_eq!(eres.sum_versions, 14);

        let data = "8A004A801A8002F478";
        let eres = test_eval(data);
        assert_eq!(eres.sum_versions, 16);

        let data = "620080001611562C8802118E34";
        let eres = test_eval(data);
        assert_eq!(eres.sum_versions, 12);

        let data = "C0015000016115A2E0802F182340";
        let eres = test_eval(data);
        assert_eq!(eres.sum_versions, 23);

        let data = "A0016C880162017C3686B18A3D4780";
        let eres = test_eval(data);
        assert_eq!(eres.sum_versions, 31);
    }

    #[test]
    fn test_p2() {
        let data = "C200B40A82";
        let eres = test_eval(data);
        assert_eq!(eres.value, 3);

        let data = "04005AC33890";
        let eres = test_eval(data);
        assert_eq!(eres.value, 54);

        let data = "880086C3E88112";
        let eres = test_eval(data);
        assert_eq!(eres.value, 7);

        let data = "CE00C43D881120";
        let eres = test_eval(data);
        assert_eq!(eres.value, 9);

        let data = "D8005AC2A8F0";
        let eres = test_eval(data);
        assert_eq!(eres.value, 1);

        let data = "F600BC2D8F";
        let eres = test_eval(data);
        assert_eq!(eres.value, 0);

        let data = "9C005AC2F8F0";
        let eres = test_eval(data);
        assert_eq!(eres.value, 0);

        let data = "9C0141080250320F1802104A08";
        let eres = test_eval(data);
        assert_eq!(eres.value, 1);
    }
}
