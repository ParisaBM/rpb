// use std::collections::HashMap;

use parlay::internal::group_by::{group_by_key, histogram_by_key};
// use parlay::internal::sample_sort_inplace;
// use parlay::primitives::flatten_by_val;
use parlay::primitives::tokens;
use parlay::Timer;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
};
use rayon::slice::ParallelSliceMut;

use crate::misc::DefChar;

pub fn djb2_hash_char(input: &[DefChar]) -> usize {
    let mut hash = 5381;

    for char in input {
        hash = ((hash << 5) + hash) + *char as usize;
    }
    hash
}

pub fn djb2_hash_str(input: &str) -> usize {
    let mut hash = 5381;

    for char in input.chars() {
        hash = ((hash << 5) + hash) + char as usize;
    }
    hash
}

pub fn build_index(s: &[DefChar], doc_start: &str, result: &mut Vec<char>) {
    let mut t = Timer::new("index");

    let n = s.len();
    let m = doc_start.len();

    let doc_start_vec: &[DefChar] = doc_start.as_bytes();

    // sequence of indices to the start of each document
    let starts: Vec<usize> = s
        .par_iter()
        .enumerate()
        .filter_map(|(i, _)| {
            for j in 0..m {
                if doc_start_vec[j] != s[i + j] {
                    return None;
                }
            }
            Some(i)
        })
        .collect();
    let num_docs = starts.len();
    t.next("get starts");

    // generate sequence of token-doc_id pairs for each document
    let docs: Vec<Vec<(String, usize)>> = (0..num_docs)
        .into_par_iter()
        .map(|doc_id: usize| {
            let start = starts[doc_id] + m;
            let end = if doc_id == num_docs - 1 {
                n
            } else {
                starts[doc_id + 1]
            };

            // blank out all non characters, and convert to lowercase
            let str: Vec<DefChar> = s[start..end]
                .par_iter()
                .map(|c| -> DefChar {
                    if *c >= b'A' && *c <= b'Z' {
                        *c + 32
                    } else if *c >= b'a' && *c <= b'z' {
                        *c
                    } else {
                        0
                    }
                })
                .collect();

            // generate tokens (i.e., contiguous regions of non-zero characters)
            let tokens: Vec<&[DefChar]> = tokens(&str, |c| *c == 0);

            // remove duplicate tokens
            let mut word_map: Vec<(&[DefChar], usize)> = Vec::new();
            // couldn't make remove_duplicates work with T=&Vec<H>
            histogram_by_key(&tokens, djb2_hash_char, &mut word_map);

            word_map
                .par_iter()
                .map(|(token, _)| {
                    let token_str: String = String::from_utf8(token.to_vec()).unwrap();
                    (token_str, doc_id)
                })
                .collect()
        })
        .collect();
    t.next("generate document tokens");

    let word_doc_pairs: Vec<(String, usize)> = docs.into_par_iter().flatten().collect();
    // let mut word_doc_pairs: Vec<(String, usize)> = Vec::new();
    // flatten_by_val(&docs, &mut word_doc_pairs);
    t.next("flatten document tokens");

    // group by word, each with a sequence of docs it appears in.
    let mut words: Vec<(&str, Vec<usize>)> = Vec::new();
    let ref_word_doc_pairs: Vec<(&str, usize)> = word_doc_pairs
        .iter()
        .map(|(k, v)| (k.as_str(), *v))
        .collect();
    group_by_key(&ref_word_doc_pairs, djb2_hash_str, &mut words);
    t.next("group by word");

    // let keys: Vec<&str> = words.iter().map(|(k, _)| *k).collect();
    // sample_sort_inplace(&mut keys_clone, |a: &str, b: &str| a < b, false);
    // let lookup: HashMap<&str, Vec<usize>> = words.into_iter().collect();
    // let sorted_words: Vec<(&str, Vec<usize>)> =
    //     keys.into_iter().map(|k| (k, lookup[&k].clone())).collect();
    words.par_sort_unstable_by_key(|&(word, _)| word);
    t.next("sort words");

    // format output for each word
    let output: Vec<char> = words
        .par_iter()
        .flat_map(|(word, doc_ids)| {
            let mut word_chars: Vec<char> = word.chars().collect();
            let mut doc_ids_chars: Vec<char> = doc_ids
                .par_iter()
                .enumerate()
                .flat_map(|(i, n)| {
                    let mut chars: Vec<char> = n.to_string().chars().collect();
                    if i != doc_ids.len() - 1 {
                        chars.push(' '); // add space between numbers
                    }
                    chars
                })
                .collect();

            word_chars.push(' ');
            word_chars.append(&mut doc_ids_chars);
            word_chars.push('\n');

            word_chars
        })
        .collect();
    *result = output;
    t.next("format words");
}
