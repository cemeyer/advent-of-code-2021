#![allow(dead_code, unused_assignments, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use bitvec::prelude::*;
use graphlib::{Graph, VertexId};
use itertools::iproduct;
//use nalgebra::*;
//use ndarray::prelude::*;
//use ndarray::{ArcArray2, parallel::par_azip};
use std::cmp::{min, max};
use std::convert::{TryFrom, TryInto};
use std::collections::*;
use std::hash::Hash;
use std::iter::FromIterator;

use aoc::{dbg2, byte, BitCursor, ByteString};

type Range = (i32, i32);
type ParseResult = (Vec<(bool, Range, Range, Range)>);

fn parse(data: &str) -> ParseResult {
    data.lines().map(|line| {
        let (onoff, ranges) = line.split_once(' ').unwrap();
        let on = if onoff == "on" { true } else if onoff == "off" { false } else { unreachable!(); };
        let (x, yz) = ranges.split_once(',').unwrap();
        let (y, z) = yz.split_once(',').unwrap();

        let mut xyz = ranges.split(',').map(|rng| {
            let rng = rng.split_once('=').unwrap().1;
            let mut rng = rng.split("..").map(|x| x.parse().unwrap());
            let a = rng.next().unwrap();
            let b = rng.next().unwrap();
            (a, b)
        });

        (on, xyz.next().unwrap(), xyz.next().unwrap(), xyz.next().unwrap())
    })
    .collect::<Vec<_>>()
}

fn part1(input: &ParseResult) -> u64 {
    let bounded = input.iter()
        .copied()
        .filter(|(_, x, y, z)| {
            (x.1 >= -50 && x.0 <= 50) &&
            (y.1 >= -50 && y.0 <= 50) &&
            (z.1 >= -50 && z.0 <= 50)
        })
        .map(|(onoff, x, y, z)| {
            (onoff,
             (max(-50, x.0), min(50, x.1)),
             (max(-50, y.0), min(50, y.1)),
             (max(-50, z.0), min(50, z.1)),
             )
    })
    .collect::<Vec<_>>();
    part2(&bounded)
}

fn part2(input: &ParseResult) -> u64 {
    let mut boxes = Vec::new();
    let mut total = 0u64;

    for (_line, instr) in input.iter().cloned().enumerate() {
        let (onoff, xr, yr, zr) = instr;
        let (xlo, xhi) = (xr.0, xr.1);
        let (ylo, yhi) = (yr.0, yr.1);
        let (zlo, zhi) = (zr.0, zr.1);

        let next = [[xlo, ylo, zlo], [xhi, yhi, zhi]];
        let nextvol = (xhi as i64 - xlo as i64 + 1) * (yhi as i64 - ylo as i64 + 1) * (zhi as i64 - zlo as i64 + 1);

        let mut rem = Vec::new();
        for existing in 0..boxes.len() {
            if let Some(overlap) = calc_overlap(&next, &boxes[existing]) {
                // Cut overlapped region out of existing box.
                delete_subcube(&mut rem, &mut boxes[existing], &overlap);
                // Subtract deleted region from total.
                let olx = (overlap[0][0] as i64, overlap[1][0] as i64);
                let oly = (overlap[0][1] as i64, overlap[1][1] as i64);
                let olz = (overlap[0][2] as i64, overlap[1][2] as i64);
                total -= ((olx.1 - olx.0 + 1) * (oly.1 - oly.0 + 1) * (olz.1 - olz.0 + 1)) as u64;
            }
        }

        boxes.append(&mut rem);
        if onoff {
            total += nextvol as u64;
            boxes.push(next);
        }
    }
    //dbg2!(boxes.len());
    //let _dead = [[i32::MIN,i32::MIN,i32::MIN],[i32::MIN,i32::MIN,i32::MIN]];
    //dbg2!(boxes.iter().filter(|b| *b == &_dead).count());
    total
}

fn delete_subcube(rem: &mut Vec<[[i32;3];2]>, haystack: &mut [[i32;3];2], needle: &[[i32;3];2]) {
    let haystack = {
        let mut stack = [[i32::MIN,i32::MIN,i32::MIN],[i32::MIN,i32::MIN,i32::MIN]];
        std::mem::swap(&mut stack, haystack);
        stack
    };

    // six (possible) sub-cubes replace haystack.

    if needle[0][0] > haystack[0][0] {
        // left side cuboid "1"
        rem.push([haystack[0], [needle[0][0] - 1, haystack[1][1], haystack[1][2]]]);
    }

    if needle[1][0] < haystack[1][0] {
        // right side cuboid "2"
        rem.push([[needle[1][0] + 1, haystack[0][1], haystack[0][2]], haystack[1]]);
    }

    if needle[0][1] > haystack[0][1] {
        // bottom side cuboid "4"
        rem.push([[needle[0][0], haystack[0][1], haystack[0][2]],
                 [needle[1][0], needle[0][1] - 1, haystack[1][2]]]);
    }

    if needle[1][1] < haystack[1][1] {
        // top side cuboid "3"
        rem.push([[needle[0][0], needle[1][1] + 1, haystack[0][2]],
                 [needle[1][0], haystack[1][1], haystack[1][2]]]);
    }

    if needle[0][2] > haystack[0][2] {
        // front cuboid "5"
        rem.push([[needle[0][0], needle[0][1], haystack[0][2]],
                 [needle[1][0], needle[1][1], needle[0][2] - 1]]);
    }

    if needle[1][2] < haystack[1][2] {
        // rear cuboid "6"
        rem.push([[needle[0][0], needle[0][1], needle[1][2] + 1],
                 [needle[1][0], needle[1][1], haystack[1][2]]]);
    }
}

#[inline]
fn calc_overlap(a: &[[i32;3];2], b: &[[i32;3];2]) -> Option<[[i32;3];2]> {
    let xhi = min(a[1][0], b[1][0]);
    let yhi = min(a[1][1], b[1][1]);
    let zhi = min(a[1][2], b[1][2]);

    let xlo = max(a[0][0], b[0][0]);
    let ylo = max(a[0][1], b[0][1]);
    let zlo = max(a[0][2], b[0][2]);

    if xhi >= xlo && yhi >= ylo && zhi >= zlo {
        Some([[xlo, ylo, zlo], [xhi, yhi, zhi]])
    } else {
        None
    }
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 22)?;
    let data = puzzle.get_data()?;
    //let data = SAMPLE_DATA;
    let parsed = parse(data);

    let answ1 = part1(&parsed);
    dbg!(&answ1);
    assert_eq!(answ1, 615700);
    let answ2 = part2(&parsed);
    dbg!(&answ2);
    assert!(answ2 > 1236364099896881);
    assert!(answ2 > 1236455829265038);
    assert_eq!(answ2, 1236463892941356);

    //puzzle.submit_answer(aoc::Part::One, &format!("{}", answ1))?;
    //puzzle.submit_answer(aoc::Part::Two, &format!("{}", answ2))?;
    Ok(())
}

const SAMPLE_DATA: &str =
"on x=-5..47,y=-31..22,z=-19..33
on x=-44..5,y=-27..21,z=-14..35
on x=-49..-1,y=-11..42,z=-10..38
on x=-20..34,y=-40..6,z=-44..1
off x=26..39,y=40..50,z=-2..11
on x=-41..5,y=-41..6,z=-36..8
off x=-43..-33,y=-45..-28,z=7..25
on x=-33..15,y=-32..19,z=-34..11
off x=35..47,y=-46..-34,z=-11..5
on x=-14..36,y=-6..44,z=-16..29
on x=-57795..-6158,y=29564..72030,z=20435..90618
on x=36731..105352,y=-21140..28532,z=16094..90401
on x=30999..107136,y=-53464..15513,z=8553..71215
on x=13528..83982,y=-99403..-27377,z=-24141..23996
on x=-72682..-12347,y=18159..111354,z=7391..80950
on x=-1060..80757,y=-65301..-20884,z=-103788..-16709
on x=-83015..-9461,y=-72160..-8347,z=-81239..-26856
on x=-52752..22273,y=-49450..9096,z=54442..119054
on x=-29982..40483,y=-108474..-28371,z=-24328..38471
on x=-4958..62750,y=40422..118853,z=-7672..65583
on x=55694..108686,y=-43367..46958,z=-26781..48729
on x=-98497..-18186,y=-63569..3412,z=1232..88485
on x=-726..56291,y=-62629..13224,z=18033..85226
on x=-110886..-34664,y=-81338..-8658,z=8914..63723
on x=-55829..24974,y=-16897..54165,z=-121762..-28058
on x=-65152..-11147,y=22489..91432,z=-58782..1780
on x=-120100..-32970,y=-46592..27473,z=-11695..61039
on x=-18631..37533,y=-124565..-50804,z=-35667..28308
on x=-57817..18248,y=49321..117703,z=5745..55881
on x=14781..98692,y=-1341..70827,z=15753..70151
on x=-34419..55919,y=-19626..40991,z=39015..114138
on x=-60785..11593,y=-56135..2999,z=-95368..-26915
on x=-32178..58085,y=17647..101866,z=-91405..-8878
on x=-53655..12091,y=50097..105568,z=-75335..-4862
on x=-111166..-40997,y=-71714..2688,z=5609..50954
on x=-16602..70118,y=-98693..-44401,z=5197..76897
on x=16383..101554,y=4615..83635,z=-44907..18747
off x=-95822..-15171,y=-19987..48940,z=10804..104439
on x=-89813..-14614,y=16069..88491,z=-3297..45228
on x=41075..99376,y=-20427..49978,z=-52012..13762
on x=-21330..50085,y=-17944..62733,z=-112280..-30197
on x=-16478..35915,y=36008..118594,z=-7885..47086
off x=-98156..-27851,y=-49952..43171,z=-99005..-8456
off x=2032..69770,y=-71013..4824,z=7471..94418
on x=43670..120875,y=-42068..12382,z=-24787..38892
off x=37514..111226,y=-45862..25743,z=-16714..54663
off x=25699..97951,y=-30668..59918,z=-15349..69697
off x=-44271..17935,y=-9516..60759,z=49131..112598
on x=-61695..-5813,y=40978..94975,z=8655..80240
off x=-101086..-9439,y=-7088..67543,z=33935..83858
off x=18020..114017,y=-48931..32606,z=21474..89843
off x=-77139..10506,y=-89994..-18797,z=-80..59318
off x=8476..79288,y=-75520..11602,z=-96624..-24783
on x=-47488..-1262,y=24338..100707,z=16292..72967
off x=-84341..13987,y=2429..92914,z=-90671..-1318
off x=-37810..49457,y=-71013..-7894,z=-105357..-13188
off x=-27365..46395,y=31009..98017,z=15428..76570
off x=-70369..-16548,y=22648..78696,z=-1892..86821
on x=-53470..21291,y=-120233..-33476,z=-44150..38147
off x=-93533..-4276,y=-16170..68771,z=-104985..-24507";
