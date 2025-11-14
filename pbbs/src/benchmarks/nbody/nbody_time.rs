#![allow(dead_code)]

#[path = "../../common/mod.rs"]
mod common;
#[path = "../macros.rs"]
mod macros;
#[path = "../../misc.rs"]
mod misc;
#[path = "mod.rs"]
mod nbody;

use misc::*;
use std::time::Duration;

use common::geometry::Point3d;
use common::geometry_io;

use crate::nbody::parallel_ck;

define_args!(Algs::ParallelCK);
define_algs!((ParallelCK, "Parallel Callahan-Kosaraju"));

fn run(alg: Algs, rounds: usize, pts: &Vec<Point3d<f64>>) -> (Vec<Point3d<f64>>, Duration) {
    let f = match alg {
        Algs::ParallelCK => parallel_ck::nbody,
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

fn main() {
    init!();

    let args = Args::parse();
    let pts = geometry_io::read_points3d_from_file::<f64>(&args.ifname);
    let (r, d) = run(args.algorithm, args.rounds, &pts);

    finalize!(
        args,
        r,
        d,
        geometry_io::write_points3d_to_file(&r, args.ofname)
    );
}
