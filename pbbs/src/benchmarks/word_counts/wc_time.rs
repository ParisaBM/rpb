use std::time::Duration;
use std::fs;

#[path ="mod.rs"] mod wc;
#[path ="../../misc.rs"] mod misc;
#[path ="../macros.rs"] mod macros;
#[path ="../../common/io.rs"] mod io;

use misc::*;
use wc::wc_serial;
use io::chars_from_file;

type ResultType = (String, usize);

define_args!(Algs::WordCounts);

define_algs!((WordCounts, "word-counts"));

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
        Algs::WordCounts => {wc_serial::word_counts},
    };

    let mut r: Vec<ResultType> = Vec::new();

    // convert u8 (DefChar) to char type
    let vec_inp: Vec<char> = inp.iter()
                                .map(|c| *c as char)
                                .collect();

    let mean = time_loop(
        "wc",
        rounds,
        Duration::new(1, 0),
        || {},
        || { f(&vec_inp, &mut r); },
        || {}
    );

    (r, mean)
}

fn main() {
    init!();
    let args = Args::parse();
    let arr: Vec<DefChar> = chars_from_file(&args.ifname, false).unwrap();

    let (r, d) = run(args.algorithm, args.rounds, &arr);

    finalize!(
        args,
        r,
        d,
        write_histograms_to_file(&r, &args.ofname)
    );
}