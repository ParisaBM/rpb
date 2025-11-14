use parlay::Timer;

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

type ResultType = (String, usize);

struct Djb2 {
    hash: u64,
}

impl Default for Djb2 {
    fn default() -> Self {
        Djb2 { hash: 5381 }
    }
}

impl Hasher for Djb2 {
    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.hash = ((self.hash << 5) + self.hash) + (*b as u64);
        }
    }

    fn finish(&self) -> u64 {
        self.hash
    }
}

type Djb2Hasher = BuildHasherDefault<Djb2>;

pub fn word_counts(s: &Vec<char>, result: &mut Vec<ResultType>) {
    // remove any previous run results from time_loop
    result.clear();

    let mut t = Timer::new("wc");

    // convert Vec<char> to a large String
    // if [A-Z], convert to lower case
    // else if [a-z], no change
    // else, convert to whitespace
    let str: String = s
        .iter()
        .map(|c| -> char {
            if c.is_ascii_uppercase() {
                c.to_ascii_lowercase()
            } else if c.is_ascii_lowercase() {
                *c
            } else {
                ' '
            }
        })
        .collect();
    t.next("copy");

    // tokenize
    let tokens: Vec<&str> = str.split_ascii_whitespace().collect();
    // println!("number of words = {}", tokens.len());
    t.next("tokenize");

    // define a hash table
    let mut word_map: HashMap<&str, usize, Djb2Hasher> =
        HashMap::with_capacity_and_hasher(tokens.len(), Djb2Hasher::default());

    // add each token to the word_map
    for token in tokens {
        word_map
            .entry(token)
            .and_modify(|counter| *counter += 1) // increment by 1 if entry exist
            .or_insert(1); // else this is first occurance
    }
    t.next("insert into hash table");

    for (token, count) in word_map {
        result.push((token.to_string(), count));
    }
    t.next("extract results");
}
