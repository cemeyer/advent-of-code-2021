#![allow(dead_code, unused_assignments, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use bitvec::prelude::*;
use graphlib::{Graph, VertexId};
use itertools::{Itertools, iproduct};
//use ndarray::prelude::*;
//use ndarray::{ArcArray2, parallel::par_azip};
//use nalgebra::{Matrix4, Point3, Transform3, Vector3};
use nalgebra::*;
use std::cmp::{min, max};
use std::convert::{TryFrom, TryInto};
use std::collections::*;
use std::hash::Hash;
use std::iter::FromIterator;

use aoc::{dbg2, byte, BitCursor, ByteString};

type Vec3f = Vector3<f64>;
type Pt3f = Point3<f64>;

type ParseResult = Vec<ScannerPoints>;

fn parse(data: &str) -> ParseResult {
    data.split("\n\n").map(|scanner| {
        let mut lines = scanner.lines();
        lines.next().unwrap();

        let points = lines.map(|line| {
            let (x, rest) = line.split_once(',').unwrap();
            let (y, z) = rest.split_once(',').unwrap();
            Pt3f::from(Vec3f::from_iterator(
                    line.split(',').flat_map(|w| w.parse())
                    ))
        }).collect::<Vec<_>>();
        ScannerPoints::new(points)
    }).collect::<Vec<_>>()
}

// algorithm from tjol
#[derive(PartialEq,Clone,Debug)]
struct ScannerPoints {
    points: Vec<Pt3f>,
    // idx from points -> { distance => [ indices from pts ] }
    sq_distance_map: Vec<HashMap<u32, Vec<usize>>>,
    uniq_dist_lists: Vec<Vec<u32>>,
}

impl ScannerPoints {
    fn new(points: Vec<Pt3f>) -> Self {
        let mut sq_distance_map = Vec::new();
        sq_distance_map.resize_with(points.len(), HashMap::new);

        for i in 0..points.len() {
            let p = &points[i];
            for j in (i + 1)..points.len() {
                let q = &points[j];
                let sq_dist = (p - q).norm_squared() as u32;
                sq_distance_map[i].entry(sq_dist)
                    .or_insert(Vec::new())
                    .push(j);
                sq_distance_map[j].entry(sq_dist)
                    .or_default()
                    .push(i);
            }
        }

        // Extract, for each point, an ordered list of unique distances to other beacons visible to
        // this scanner.
        let uniq_dist_lists = sq_distance_map.iter()
            .map(|m| {
                m.iter()
                    .filter_map(|(k, v)| if v.len() == 1 { Some(k) } else { None })
                    .copied()
                    .sorted()
                    .collect()
            })
        .collect();

        Self {
            points,
            sq_distance_map,
            uniq_dist_lists,
        }
    }

    // Returns pairs of indices of matching beacons, if there were sufficient distance matches to
    // declare this a match.
    //
    // Vec<(idx in self.points, idx in other.points)>
    fn match_with(&self, other: &Self) -> Option<Vec<(usize, usize)>> {
        // We're actually looking through each pair of (my, other) *beacons* to find ones with
        // enough relative-distance unique matches.
        for (i, my_dists) in self.uniq_dist_lists.iter().enumerate() {
            for (j, their_dists) in other.uniq_dist_lists.iter().enumerate() {
                let (mut k, mut l) = (0, 0);
                let mut matches = Vec::new();

                // Walk both (sorted) vectors forward to find matching distances.
                while k < my_dists.len() && l < their_dists.len() {
                    let myval = my_dists[k];
                    let thval = their_dists[l];

                    if myval < thval {
                        k += 1;
                    } else if myval > thval {
                        l += 1;
                    } else {
                        //assert_eq!(myval, thval);
                        let a = self.sq_distance_map[i][&myval][0];
                        let b = other.sq_distance_map[j][&thval][0];

                        matches.push((a, b));
                        k += 1;
                        l += 1;
                    }
                }

                // Arbitrarily lower threshold than specified 12, to account for removal of
                // non-unique distances earlier.
                if matches.len() >= 11 {
                    matches.insert(0, (i, j));
                    return Some(matches);
                }
            }
        }
        None
    }

    // This is the magic.
    fn get_transformation_onto(&self, other: &Self) -> Option<Transform3<f64>> {
        let matches = &self.match_with(other)?[..4];

        // Oh, clever.  Need 4 points to make a unique transformation in 3d space, I guess.
        let their_mat = Matrix4::from_columns(
            &matches.iter()
                .map(|(_, j)| other.points[*j].to_homogeneous())
                .collect::<Vec<_>>()
        );
        let mut our_mat = Matrix4::from_columns(
            &matches.iter()
                .map(|(i, _)| self.points[*i].to_homogeneous())
                .collect::<Vec<_>>()
        );
        if !our_mat.try_inverse_mut() {
            unreachable!();
        }

        Some(Transform3::from_matrix_unchecked(their_mat * our_mat))
    }
}

fn get_abs_transformations(scanner_points: &ParseResult) -> HashMap<usize, Transform3<f64>> {
    // scanner idx -> transformation from viewpoint of first scanner
    let mut transformations = HashMap::new();
    transformations.insert(0, Transform3::identity());
    let mut unsolved_scanners = HashSet::<usize>::from_iter((1..scanner_points.len()));
    //let mut unsolved_scanners: Vec::<usize> = (1..scanner_points.len()).collect();
    // Pairs of scanners we've already tried (avoid repeating work):
    let mut checked = HashSet::new();

    // Try and link each remaining scanner into the graph, until there are no matches between the
    // unsolved set and any known graph component.
    while !unsolved_scanners.is_empty() {
        for (_unsolved_idx, i) in unsolved_scanners.clone().into_iter().enumerate() {
            let mut found_any = false;

            let known = transformations.keys().collect::<Vec<_>>();
            for j in known {
                // Avoid repeating known-bad pairs.
                if checked.contains(&(i, *j)) {
                    continue;
                }

                if let Some(trans) = scanner_points[i].get_transformation_onto(&scanner_points[*j]) {
                    // Compute scanner0-relative ("absolute") transformation from 0->j and j->i
                    // transformations.
                    let trans = transformations[j] * trans;
                    transformations.insert(i, trans);
                    unsolved_scanners.remove(&i);
                    //unsolved_scanners.swap_remove(_unsolved_idx);
                    found_any = true;
                    break;
                } else {
                    checked.insert((i, *j));
                }
            }

            if found_any {
                break;
            }
        }
    }

    // Apparently, all scanners are in the graph.
    //dbg!(unsolved_scanners.len());
    assert!(unsolved_scanners.is_empty());

    transformations
}

fn part1(scanner_points: &ParseResult, transformations: &HashMap<usize, Transform3<f64>>) -> usize {
    // Finally, dedupe beacons.
    let mut beacons = HashSet::new();
    for i in 0..scanner_points.len() {
        let trans = transformations[&i];
        for q in scanner_points[i].points.iter() {
            let q = trans * q;
            let q_int = Point3::from(Vector3::<i32>::from_iterator(
                    q.coords.iter().map(|f| f.round() as i32)));
            //dbg!(q_int);
            beacons.insert(q_int);
        }
    }

    // debugging f32 lack of precision issue!
    //for b in beacons.iter()
    //    .map(|b| (b[0], b[1], b[2]))
    //    .sorted() {
    //    println!("{},{},{}", b.0, b.1, b.2);
    //}

    beacons.len()
}

fn part2(scanner_points: &ParseResult, transformations: &HashMap<usize, Transform3<f64>>) -> i32 {
    let scanner_locs = (0..scanner_points.len()).map(|i| {
        let origin = transformations[&i] * point![0.,0.,0.];
        Point3::from(Vector3::<i32>::from_iterator(
                origin.coords.iter().map(|f| f.round() as i32)))
    })
    .collect::<Vec<_>>();

    let mut max = 0;
    for (i, loc1) in scanner_locs.iter().enumerate() {
        for loc2 in &scanner_locs[i + 1..] {
            let dist = (loc2 - loc1).abs().sum();
            if dist > max {
                max = dist;
            }
        }
    }
    max
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 19)?;
    let data = puzzle.get_data()?;
    //let data = SAMPLE_DATA;
    let parsed = parse(data);

    let transformations = get_abs_transformations(&parsed);

    let answ1 = part1(&parsed, &transformations);
    dbg!(answ1);
    //assert!(answ1 < 390);
    assert_eq!(answ1, 378);

    let answ2 = part2(&parsed, &transformations);
    dbg!(answ2);
    assert_eq!(answ2, 13148);

    Ok(())
}
//const SAMPLE_DATA: &str =
//"--- scanner 0 ---
//404,-588,-901
//528,-643,409
//-838,591,734
//390,-675,-793
//-537,-823,-458
//-485,-357,347
//-345,-311,381
//-661,-816,-575
//-876,649,763
//-618,-824,-621
//553,345,-567
//474,580,667
//-447,-329,318
//-584,868,-557
//544,-627,-890
//564,392,-477
//455,729,728
//-892,524,684
//-689,845,-530
//423,-701,434
//7,-33,-71
//630,319,-379
//443,580,662
//-789,900,-551
//459,-707,401
//
//--- scanner 1 ---
//686,422,578
//605,423,415
//515,917,-361
//-336,658,858
//95,138,22
//-476,619,847
//-340,-569,-846
//567,-361,727
//-460,603,-452
//669,-402,600
//729,430,532
//-500,-761,534
//-322,571,750
//-466,-666,-811
//-429,-592,574
//-355,545,-477
//703,-491,-529
//-328,-685,520
//413,935,-424
//-391,539,-444
//586,-435,557
//-364,-763,-893
//807,-499,-711
//755,-354,-619
//553,889,-390
//
//--- scanner 2 ---
//649,640,665
//682,-795,504
//-784,533,-524
//-644,584,-595
//-588,-843,648
//-30,6,44
//-674,560,763
//500,723,-460
//609,671,-379
//-555,-800,653
//-675,-892,-343
//697,-426,-610
//578,704,681
//493,664,-388
//-671,-858,530
//-667,343,800
//571,-461,-707
//-138,-166,112
//-889,563,-600
//646,-828,498
//640,759,510
//-630,509,768
//-681,-892,-333
//673,-379,-804
//-742,-814,-386
//577,-820,562
//
//--- scanner 3 ---
//-589,542,597
//605,-692,669
//-500,565,-823
//-660,373,557
//-458,-679,-417
//-488,449,543
//-626,468,-788
//338,-750,-386
//528,-832,-391
//562,-778,733
//-938,-730,414
//543,643,-506
//-524,371,-870
//407,773,750
//-104,29,83
//378,-903,-323
//-778,-728,485
//426,699,580
//-438,-605,-362
//-469,-447,-387
//509,732,623
//647,635,-688
//-868,-804,481
//614,-800,639
//595,780,-596
//
//--- scanner 4 ---
//727,592,562
//-293,-554,779
//441,611,-461
//-714,465,-776
//-743,427,-804
//-660,-479,-426
//832,-632,460
//927,-485,-438
//408,393,-506
//466,436,-512
//110,16,151
//-258,-428,682
//-393,719,612
//-211,-452,876
//808,-476,-593
//-575,615,604
//-485,667,467
//-680,325,-822
//-627,-443,-432
//872,-547,-609
//833,512,582
//807,604,487
//839,-516,451
//891,-625,532
//-652,-548,-490
//30,-46,-14";
