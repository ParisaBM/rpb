#![allow(dead_code)]

#[path = "../../common/mod.rs"]
mod common;
#[path = "../macros.rs"]
mod macros;
#[path = "../../misc.rs"]
mod misc;
#[path = "mod.rs"]
mod nearest_neighbor;

use misc::*;
use std::time::Duration;

use crate::common::{geometry::Point, geometry_io, io::write_slice_to_file_seq};

define_args!(Algs::Naive, (dimension, usize, 2));
define_algs!((Naive, "naive"), (Chan05, "chan05"));

fn run<const D: usize>(
    alg: Algs,
    rounds: usize,
    pts: &Vec<Point<D, f64>>,
) -> (Vec<usize>, Duration) {
    let f = match alg {
        Algs::Naive => nearest_neighbor::naive::knn,
        Algs::Chan05 => nearest_neighbor::chan05::knn,
    };

    let mut r = Vec::new();
    let mean = time_loop(
        "nbody",
        rounds,
        Duration::new(1, 0),
        || {},
        || r = f(pts),
        || {},
    );

    (r, mean)
}

fn handle_args<const D: usize>(args: Args) {
    let pts = geometry_io::read_points_from_file::<D, f64>(&args.ifname);
    let (r, d) = run(args.algorithm, args.rounds, &pts);

    finalize!(args, r, d, write_slice_to_file_seq(&r, args.ofname));
}

fn main() {
    init!();

    let args = Args::parse();
    if args.dimension == 2 {
        handle_args::<2>(args);
    } else if args.dimension == 3 {
        handle_args::<3>(args);
    } else {
        panic!("Dimension must be 2 or 3");
    }
}
