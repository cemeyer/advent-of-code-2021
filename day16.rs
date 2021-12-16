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

#[derive(Debug,Clone)]
struct Packet {
    version: u8,
    type_: u8,
    value: PVal,
}

#[derive(Debug,Clone)]
enum PVal {
    Literal(u64),
    Operator,
}

fn eval22(input: &BitSlice::<Msb0, u8>) -> (u64, usize) {
    let version = input[0..3].load_be::<u8>();
    let type_ = input[3..6].load_be::<u8>();

    let mut pktlen = 6;

    let mut cursor = &input[6..];
    let pktval = match type_ {
        0x4 => {
            let mut res = 0u64;
            loop {
                let next = cursor[0..5].load_be::<u8>();
                cursor = &cursor[5..];
                pktlen += 5;

                res = res << 4;
                res |= (next & 0xf) as u64;

                if (next & 0x10) == 0 {
                    break;
                }
            }
            res
        }
        _ => {
            // operator
            let lentype = cursor[0];
            cursor = &cursor[1..];
            pktlen += 1;

            let mut subpackets = Vec::new();

            match lentype {
                false => {
                    // total len of subpackets in bits
                    let tlen = cursor[0..15].load_be::<u16>();
                    cursor = &cursor[15..];
                    pktlen += 15;

                    let mut sublen = 0;
                    while sublen < tlen as usize {
                        let (subp, len) = eval22(cursor);
                        subpackets.push(subp);
                        sublen += len;
                        pktlen += len;
                        cursor = &cursor[len..];
                    }
                    assert_eq!(sublen, tlen as usize);
                }
                true => {
                    // number of subpackets
                    let nsubpackets = cursor[0..11].load_be::<u16>();
                    cursor = &cursor[11..];
                    pktlen += 11;

                    for _i in 0..nsubpackets {
                        let (subp, len) = eval22(cursor);
                        subpackets.push(subp);
                        cursor = &cursor[len..];
                        pktlen += len;
                    }
                }
            }

            match type_ {
                0 => subpackets.iter().sum::<u64>(),
                1 => subpackets.iter().product(),
                2 => *subpackets.iter().min().unwrap(),
                3 => *subpackets.iter().max().unwrap(),
                5 => if subpackets[0] > subpackets[1] { 1 } else { 0 },
                6 => if subpackets[0] < subpackets[1] { 1 } else { 0 },
                7 => if subpackets[0] == subpackets[1] { 1 } else { 0 },
                _ => { unreachable!(); }
            }

        }
    };

    (pktval, pktlen)
}

fn part2(input: &ParseResult) -> u64 {
    let bits = input.view_bits::<Msb0>();
    let (msg, _len) = eval22(bits);
    msg
}

fn parse2(input: &BitSlice::<Msb0, u8>) -> (u64, Packet, usize) {
    let version = input[0..3].load_be::<u8>();
    let type_ = input[3..6].load_be::<u8>();

    let mut pktlen = 6;
    let mut version_tot = version as u64;

    let mut cursor = &input[6..];
    let pkt = match type_ {
        0x4 => {
            let mut res = 0u64;
            loop {
                let next = cursor[0..5].load_be::<u8>();
                cursor = &cursor[5..];
                pktlen += 5;

                res = res << 4;
                res |= (next & 0xf) as u64;

                if (next & 0x10) == 0 {
                    break;
                }
            }
            Packet {
                version,
                type_,
                value: PVal::Literal(res),
            }
        }
        _ => {
            // operator
            let lentype = cursor[0];
            cursor = &cursor[1..];
            pktlen += 1;

            match lentype {
                false => {
                    // total len of subpackets in bits
                    let tlen = cursor[0..15].load_be::<u16>();
                    cursor = &cursor[15..];
                    pktlen += 15;

                    let mut sublen = 0;
                    while sublen < tlen as usize {
                        let (versions, _subp, len) = parse2(cursor);
                        version_tot += versions;
                        sublen += len;
                        pktlen += len;
                        cursor = &cursor[len..];
                    }
                    assert_eq!(sublen, tlen as usize);

                    Packet {
                        version,
                        type_,
                        value: PVal::Operator,
                    }
                }
                true => {
                    // number of subpackets
                    let subpackets = cursor[0..11].load_be::<u16>();
                    cursor = &cursor[11..];
                    pktlen += 11;

                    for _i in 0..subpackets {
                        let (versions, _subp, len) = parse2(cursor);
                        version_tot += versions;
                        cursor = &cursor[len..];
                        pktlen += len;
                    }

                    Packet {
                        version,
                        type_,
                        value: PVal::Operator,
                    }
                }
            }
        }
    };

    (version_tot, pkt, pktlen)
}

fn part1(input: &ParseResult) -> u64 {
    let bits = input.view_bits::<Msb0>();
    let (versions, _msg, _len) = parse2(bits);
    dbg!(_msg);
    versions
}

type ParseResult = Vec<u8>;

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
