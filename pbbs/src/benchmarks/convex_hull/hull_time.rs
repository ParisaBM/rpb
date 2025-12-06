#![allow(dead_code)]

#[path = "../../common/mod.rs"]
mod common;
#[path = "mod.rs"]
mod convex_hull;
#[path = "../macros.rs"]
mod macros;
#[path = "../../misc.rs"]
mod misc;

use misc::*;
use std::time::Duration;

use crate::common::{geometry::Point, geometry_io, io::write_slice_to_file_seq};

define_args!(Algs::Sequential);
define_algs!((Parallel, "parallel"), (Sequential, "sequential"));

fn run(alg: Algs, rounds: usize, pts: &Vec<Point<2, f64>>) -> (Vec<usize>, Duration) {
    let f = match alg {
        Algs::Sequential => convex_hull::sequential::hull,
        Algs::Parallel => convex_hull::parallel::hull,
    };

    let mut r = Vec::new();
    let mean = time_loop(
        "hull",
        rounds,
        Duration::new(1, 0),
        || {},
        || r = f(pts),
        || {},
    );

    (r, mean)
}

fn main() {
    init!();

    let args = Args::parse();
    let pts = geometry_io::read_points_from_file::<2, f64>(&args.ifname);
    let (r, d) = run(args.algorithm, args.rounds, &pts);
    finalize!(args, r, d, write_slice_to_file_seq(&r, args.ofname));
}
