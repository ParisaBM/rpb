use std::path::Path;
// ============================================================================
// This code is part of RPB.
// ----------------------------------------------------------------------------
// MIT License
//
// Copyright (c) 2023-present Javad Abdi, Mark C. Jeffrey
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
// ============================================================================

use rayon::prelude::*;

#[derive(Default, Clone, Copy)]
pub struct Edge {
    pub target: usize,
}

pub struct Graph {
    n: usize,
    m: usize,
    pub nodes: Vec<usize>,
    pub edges: Vec<Edge>,
}

impl Graph {
    pub fn num_nodes(&self) -> usize {
        self.n
    }
    pub fn num_edges(&self) -> usize {
        self.m
    }

    pub fn new(num_nodes: usize, num_edges: usize) -> Self {
        Self {
            nodes: vec![0; num_nodes],
            edges: vec![Edge::default(); num_edges],
            n: num_nodes,
            m: num_edges,
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let contents = std::fs::read_to_string(path).unwrap();

        let contents = contents.splitn(4, '\n').collect::<Vec<_>>();
        let ft = contents[0];
        assert!(ft == "WeightedAdjacencyGraph" || ft == "AdjacencyGraph");
        let num_nodes = contents[1].parse().unwrap();
        let num_edges = contents[2].parse().unwrap();
        let contents = contents[3];

        let raw_nums = contents
            .par_lines()
            .map(|l| l.parse().unwrap_or(l.parse::<f64>().unwrap() as usize))
            .collect::<Vec<_>>();

        assert!(
            raw_nums.len() == num_nodes + num_edges || raw_nums.len() == num_nodes + 2 * num_edges
        );

        let mut nodes = raw_nums[..num_nodes].to_vec();
        nodes.push(num_edges);

        if raw_nums.len() == num_nodes + 2 * num_edges {
            eprintln!("Warning: graph is weighted, ignoring weights");
        };

        let edges = raw_nums[num_nodes..num_nodes + num_edges]
            .par_iter()
            .map(|t| Edge { target: *t })
            .collect();

        Self {
            nodes,
            edges,
            n: num_nodes,
            m: num_edges,
        }
    }
}
