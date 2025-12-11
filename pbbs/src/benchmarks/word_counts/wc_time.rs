use std::fs;
use std::time::Duration;
use std::cell::RefCell;

#[path = "../../common/io.rs"]
mod io;
#[path = "../macros.rs"]
mod macros;
#[path = "../../misc.rs"]
mod misc;
#[path = "mod.rs"]
mod wc;

use io::chars_from_file;
use misc::*;
use wc::{wc_histogram, wc_serial};

type ResultType = (String, usize);

define_args!(Algs::Serial);

define_algs!((Serial, "serial"), (Histogram, "histogram"));

fn write_histograms_to_file(result: &Vec<ResultType>, out_file: &String) {
    let mut output: String = String::new();

    for (token, count) in result {
        let str = format!("({token}, {count})\n");
        output.push_str(&str);
    }

    fs::write(out_file, output).unwrap();
}

pub fn run(alg: Algs, rounds: usize, inp: &[DefChar]) -> (Vec<ResultType>, Duration) {
    let f = match alg {
        Algs::Serial => wc_serial::word_counts,
        Algs::Histogram => wc_histogram::word_counts,
    };

    let r: RefCell<Vec<ResultType>> = RefCell::new(Vec::new());

    // convert u8 (DefChar) to char type
    // let vec_inp: Vec<char> = inp.iter().map(|c| *c as char).collect();

    let mean = time_loop(
        "wc",
        rounds,
        Duration::new(1, 0),
        || { r.borrow_mut().clear(); },
        || {
            f(&inp, &mut r.borrow_mut());
        },
        || {},
    );

    (r.into_inner(), mean)
}

fn main() {
    init!();
    let args = Args::parse();
    let arr: Vec<DefChar> = chars_from_file(&args.ifname, false).unwrap();

    let (r, d) = run(args.algorithm, args.rounds, &arr);

    finalize!(args, r, d, write_histograms_to_file(&r, &args.ofname));
}
