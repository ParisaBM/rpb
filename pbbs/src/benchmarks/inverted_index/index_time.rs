use std::cell::RefCell;
use std::time::Duration;

#[path = "mod.rs"]
mod index;
#[path = "../../common/io.rs"]
mod io;
#[path = "../macros.rs"]
mod macros;
#[path = "../../misc.rs"]
mod misc;

use index::index_serial;
use io::{chars_from_file, chars_to_file};
use misc::*;

use crate::index::index_parallel;

define_args!(Algs::Serial);

define_algs!((Serial, "serial"), (Parallel, "parallel"));

pub fn run(alg: Algs, s: &[DefChar], start: &str, rounds: usize) -> (Vec<char>, Duration) {
    let f = match alg {
        Algs::Serial => index_serial::build_index,
        Algs::Parallel => index_parallel::build_index,
    };

    // convert u8 (DefChar) to char type
    // let vec_s: Vec<char> = s.iter().map(|c| *c as char).collect();

    let r: RefCell<Vec<char>> = RefCell::new(Vec::new());

    let mean = time_loop(
        "index",
        rounds,
        Duration::new(2, 0),
        || {
            r.borrow_mut().clear();
        },
        || {
            f(&s, start, &mut r.borrow_mut());
        },
        || {},
    );

    (r.into_inner(), mean)
}

fn main() {
    init!();
    let args = Args::parse();
    let arr: Vec<DefChar> = chars_from_file(&args.ifname, false).unwrap();
    let header = "<doc";

    let (r, d) = run(args.algorithm, &arr, header, args.rounds);

    let r_u8: Vec<DefChar> = r.iter().map(|c| *c as DefChar).collect();
    finalize!(args, r, d, chars_to_file(&r_u8, &args.ofname).unwrap());
}
