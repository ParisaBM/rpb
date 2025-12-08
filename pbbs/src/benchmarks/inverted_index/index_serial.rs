use parlay::Timer;

use std::collections::HashMap;

// have the same complexity as C++ std::search
// https://en.cppreference.com/w/cpp/algorithm/search.html
pub fn search(input: &[char], delim: &[char]) -> Option<usize> {
    if delim.len() == 0 {
        return Some(0);
    }
    if delim.len() > input.len() {
        return None;
    }

    for i in 0..=input.len() - delim.len() {
        if input[i..i + delim.len()] == delim[..] {
            return Some(i);
        }
    }
    Some(input.iter().len()) // return out-of-bound index
}

pub fn build_index(s: &Vec<char>, doc_start: &str, result: &mut Vec<char>) {
    let mut t = Timer::new("index");

    // let n = s.len();
    let m = doc_start.len();

    // group by word, each with a sequence of docs it appears in.
    let mut words: HashMap<Vec<char>, Vec<u32>> = HashMap::new();
    // let mut doc_id_str: Vec<Vec<char>> = Vec::new();

    // Find the first document delimiter
    let doc_start_vec: Vec<char> = doc_start.chars().collect();
    let mut doc_begin = search(s, &doc_start_vec).unwrap();

    // Generate, for each document, the tokens contained within it
    let mut doc_id = 0;
    while doc_begin != s.len() {
        // doc_id_str.push(vec![doc_id as char]);

        // Find the end of the current document
        let doc_end = search(&s[doc_begin + m..], &doc_start_vec).unwrap() + doc_begin + m;

        // generate tokens (i.e., contiguous regions of alphabetic characters)
        let mut token_begin = doc_begin + m;
        // skip non-alphabetic chars beginning of a doc
        while token_begin != doc_end && !s[token_begin].is_ascii_alphabetic() {
            token_begin += 1;
        }

        while token_begin != doc_end {
            // c++ find_if
            let token_end = s[token_begin..doc_end]
                .iter()
                .position(|c| !c.is_ascii_alphabetic())
                .unwrap_or(s[token_begin..doc_end].len())
                + token_begin;

            let token: Vec<char> = s[token_begin..token_end]
                .iter()
                .map(|c| c.to_ascii_lowercase())
                .collect();

            let doc_ids_for_token = words.entry(token.clone()).or_default();
            if !doc_ids_for_token.contains(&doc_id) {
                doc_ids_for_token.push(doc_id);
            }

            token_begin = token_end;
            // skip non-alphabetic chars in between tokens
            while token_begin != doc_end && !s[token_begin].is_ascii_alphabetic() {
                token_begin += 1;
            }
        }

        doc_begin = doc_end;
        doc_id += 1;
    }
    t.next("generate document tokens");

    let mut sorted_words: Vec<&Vec<char>> = words.keys().collect();
    sorted_words.sort();

    for word in sorted_words {
        result.append(&mut word.clone());
        result.push(' ');
        for (i, doc_id) in words.get(word).unwrap().iter().enumerate() {
            result.extend(doc_id.to_string().chars());
            if i < words.get(word).unwrap().len() - 1 {
                result.push(' ');
            }
        }
        result.push('\n');
    }
    t.next("format words");
}
