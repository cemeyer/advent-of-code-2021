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

fn isbig(x: &str) -> bool {
    x.chars().next().unwrap().is_ascii_uppercase()
}

fn formatpath(g: &Graph<String>, pathv: &Vec<VertexId>) -> String {
    let mut res = Vec::new();
    for v in pathv.iter() {
        res.push(g.fetch(v).unwrap().clone());
    }
    res.join(",")
}

fn dfs(points: &mut HashMap<String, VertexId>, graph: &mut Graph<String>, path: &mut HashMap<VertexId, u32>, pathv: &mut Vec<VertexId>, pt: &VertexId) -> u64 {
    if pt == &points["end"] {
        println!("{}", formatpath(graph, pathv));
        return 1;
    }

    let mut res = 0;
    let neighbors = graph.neighbors(pt).cloned().collect::<Vec<_>>();
    for n in neighbors {
        // can repeat big caves only
        if path.contains_key(&n) {
            let label = graph.fetch(&n).unwrap();
            if !isbig(label) {
                continue;
            }
        }

        path.insert(n.clone(), path.get(&n).unwrap_or(&0) + 1);
        pathv.push(n.clone());
        res += dfs(points, graph, path, pathv, &n);
        pathv.pop();
        path.insert(n, path[&n] - 1);
        if path[&n] == 0 {
            path.remove(&n);
        }
    }
    res
}

fn part1(points: &mut HashMap<String, VertexId>, graph: &mut Graph<String>) -> u64 {
    let mut path = HashMap::new();
    let mut pathv = Vec::new();
    let start = points["start"];
    pathv.push(points["start"]);
    path.insert(points["start"], 1);
    dfs(points, graph, &mut path, &mut pathv, &start)
}

fn dfs2(points: &mut HashMap<String, VertexId>, graph: &mut Graph<String>, using2x: &mut bool, path: &mut HashMap<VertexId, u32>, pathv: &mut Vec<VertexId>, pt: &VertexId) -> u64 {
    if pt == &points["end"] {
        //println!("{}", formatpath(graph, pathv));
        return 1;
    }

    let mut res = 0;
    let neighbors = graph.neighbors(pt).cloned().collect::<Vec<_>>();
    for n in neighbors {
        let mut kill_2x = false;
        // can repeat big caves only
        if path.contains_key(&n) {
            let label = graph.fetch(&n).unwrap();
            // can actually repeat ONE small cave twice
            if !isbig(label) {
                if label == "start" {
                    continue;
                }
                if *using2x {
                    continue;
                }
                kill_2x = true;
                *using2x = true;
            }
        }

        path.insert(n.clone(), path.get(&n).unwrap_or(&0) + 1);
        pathv.push(n.clone());
        res += dfs2(points, graph, using2x, path, pathv, &n);
        if kill_2x {
            *using2x = false;
        }
        pathv.pop();
        path.insert(n, path[&n] - 1);
        if path[&n] == 0 {
            path.remove(&n);
        }
    }
    res
}

fn part2(points: &mut HashMap<String, VertexId>, graph: &mut Graph<String>) -> u64 {
    let mut path = HashMap::new();
    let mut pathv = Vec::new();
    let start = points["start"];
    let mut using2x = false;
    pathv.push(points["start"]);
    path.insert(points["start"], 1);
    dfs2(points, graph, &mut using2x, &mut path, &mut pathv, &start)
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 12)?;
    let data = puzzle.get_data()?;
//    let data =
//"dc-end
//HN-start
//start-kj
//dc-start
//dc-HN
//LN-dc
//HN-end
//kj-sa
//kj-HN
//kj-dc";
//    let data =
//"start-A
//start-b
//A-c
//A-b
//b-d
//A-end
//b-end";
    let lines = data.lines().collect::<Vec<_>>();

    let mut graph = Graph::new();
    let mut points = HashMap::new();
    for line in lines.iter() {
        let mut words = line.split('-');
        let pt1 = words.next().unwrap();
        let pt2 = words.next().unwrap();

        if !points.contains_key(pt1) {
            let v = graph.add_vertex(pt1.to_owned());
            points.insert(pt1.to_owned(), v);
        }
        if !points.contains_key(pt2) {
            let v = graph.add_vertex(pt2.to_owned());
            points.insert(pt2.to_owned(), v);
        }

        graph.add_edge(&points[pt1], &points[pt2]).unwrap();
        graph.add_edge(&points[pt2], &points[pt1]).unwrap();
    }

    //let answ1 = part1(&mut points, &mut graph);
    //dbg!(&answ1);
    let answ2 = part2(&mut points, &mut graph);
    dbg!(&answ2);

    Ok(())
}
