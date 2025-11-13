use parlay::Timer;
use parlay::internal::group_by::histogram_by_key;
use parlay::primitives::tokens;

use rayon::prelude::*;

type ResultType = (String, usize);

pub fn word_counts (s: &Vec<char>, result: &mut Vec<ResultType>) {
    // remove any previous run results from time_loop
    result.clear();

    let mut t = Timer::new("wc");

    // convert Vec<char> to a large String
    // if [A-Z], convert to lower case
    // else if [a-z], no change
    // else, convert to whitespace
    let str: Vec<char> = s.par_iter()
                            .map(|c| -> char {
                                if c.is_ascii_uppercase() { c.to_ascii_lowercase() }
                                else if c.is_ascii_lowercase() { *c }
                                else { ' ' }})
                            .collect();
    t.next("copy");

    // tokenize
    let tokens: Vec<&[char]> = tokens(&str, |c| c.is_whitespace());
    t.next("tokenize");

    // (token, count)
    let mut word_map: Vec<(&[char], usize)> = Vec::new();

    let djb2_hash = |input: &[char]| {
        let mut hash = 5381;

        for char in input {
            hash = (hash << 5 + hash) + *char as usize;
        }
        hash
    };

    histogram_by_key(&tokens, djb2_hash, &mut word_map);
    t.next("count by key");

    for (token, count) in word_map {
        let token_str: String = token.iter().collect();
        result.push((token_str, count));
    }
    t.next("extract results");

}