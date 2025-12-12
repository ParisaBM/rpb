use parlay::internal::group_by::histogram_by_key;
use parlay::primitives::tokens;
use parlay::Timer;

use rayon::prelude::*;

use crate::misc::DefChar;

type ResultType = (String, usize);

pub fn word_counts(s: &[DefChar], result: &mut Vec<ResultType>) {
    let mut t = Timer::new("wc");

    // convert Vec<char> to a large String
    // if [A-Z], convert to lower case
    // else if [a-z], no change
    // else, convert to whitespace
    let str: Vec<DefChar> = s
        .par_iter()
        .map(|c| -> DefChar {
            if *c >= b'A' && *c <= b'Z' {
                *c + 32
            } else if *c >= b'a' && *c <= b'z' {
                *c
            } else {
                b' '
            }
        })
        .collect();
    t.next("copy");

    // tokenize
    let tokens: Vec<&[DefChar]> = tokens(&str, |c| *c == b' ');
    t.next("tokenize");

    // (token, count)
    let mut word_map: Vec<(&[DefChar], usize)> = Vec::new();

    let djb2_hash = |input: &[DefChar]| {
        let mut hash = 5381;

        for char in input {
            hash = ((hash << 5) + hash) + *char as usize;
        }
        hash
    };

    histogram_by_key(&tokens, djb2_hash, &mut word_map);
    t.next("count by key");

    let output: Vec<ResultType> = word_map
        .par_iter()
        .map(|(token, count)| -> ResultType {
            let token_str: String = String::from_utf8_lossy(token).to_string();
            (token_str, *count)
        })
        .collect();
    *result = output;
    t.next("extract results");
}
