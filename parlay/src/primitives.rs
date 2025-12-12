use rayon::prelude::*;
// ============================================================================
// This code is part of RPB.
// ----------------------------------------------------------------------------
// MIT License
//
// Copyright (c) 2023-present Javad Abdi, Mark C. Jeffrey
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
// ============================================================================

use enhanced_rayon::prelude::*;
use num_traits::PrimInt;

use crate::internal::sequence_ops::*;
use crate::maybe_uninit_vec;

/* -------------------- Pack -------------------- */
fn pack_serial_at<T, F>(arr_f: F, flags: &[bool], dest: &mut [T]) -> usize
where
    T: Copy,
    F: Fn(usize) -> T,
{
    let mut k = 0;
    let n = flags.len();
    for i in 0..n {
        if flags[i] {
            dest[k] = arr_f(i);
            k += 1;
        }
    }
    k
}

pub fn pack_serial<T, F>(arr_f: F, flags: &[bool], dest: &mut Vec<T>)
where
    T: Copy + Clone,
    F: Fn(usize) -> T,
{
    let m = sum_bool_serial(flags);
    *dest = vec![arr_f(0); m];
    pack_serial_at(arr_f, flags, dest);
}

fn pack_helper<T, F>(arr_f: F, flags: &[bool], dest: &mut Vec<T>)
where
    T: Copy + Clone + Send + Sync,
    F: Fn(usize) -> T + Send + Sync,
{
    let n = flags.len();
    let bls = _BLOCK_SIZE * 10;
    let block_no = num_blocks(n, bls);
    if block_no == 1 {
        pack_serial(arr_f, flags, dest);
        return;
    }

    let mut sums: Vec<usize> = flags
        .par_chunks(bls)
        .map(|chunk| sum_bool_serial(chunk))
        .collect();

    let m = scan_inplace(&mut sums, false, |a, b| a + b);

    *dest = maybe_uninit_vec![arr_f(0); m];

    dest.par_ind_chunks_mut(&sums)
        .zip(flags.par_chunks(bls))
        .enumerate()
        .for_each(|(i, (out_chunk, flag_chunk))| {
            let s = i * bls;
            let arr_slice_f = |i| arr_f(s + i);
            pack_serial_at(arr_slice_f, flag_chunk, out_chunk);
        });
}

pub fn pack<T>(arr: &[T], flags: &[bool], dest: &mut Vec<T>)
where
    T: Copy + Send + Sync + Clone,
{
    if arr.len() > 0 {
        let arr_f = |i| arr[i];
        pack_helper(arr_f, flags, dest);
    } else {
        *dest = vec![];
    }
}

pub fn pack_index<T>(flags: &[bool], dest: &mut Vec<T>)
where
    T: Copy + Send + Sync + Clone + PrimInt,
{
    debug_assert_ne!(flags.len(), 0);
    let arr_f = |i| T::from(i).expect("pack_index: invalid conversion");
    pack_helper(arr_f, flags, dest);
}

// non copy version of pack:
// =========================
unsafe fn nc_pack_serial_at<T, F>(arr_f: F, flags: &[bool], dest: &mut [T])
where
    F: Fn(usize, *mut T),
{
    let mut k = 0;
    for i in 0..flags.len() {
        if flags[i] {
            arr_f(i, &mut dest[k]);
            k += 1;
        }
    }
}

pub unsafe fn nc_pack_serial<T, F>(arr_f: F, flags: &[bool], dest: &mut Vec<T>)
where
    F: Fn(usize, *mut T),
{
    let m = sum_bool_serial(flags);
    *dest = Vec::with_capacity(m);
    dest.set_len(m);
    nc_pack_serial_at(arr_f, flags, dest);
}

unsafe fn nc_pack_helper<T, F>(arr_f: F, flags: &[bool], dest: &mut Vec<T>)
where
    T: Send + Sync,
    F: Fn(usize, *mut T) + Send + Sync,
{
    let n = flags.len();
    let bls = _BLOCK_SIZE * 10;
    let block_no = num_blocks(n, bls);
    if block_no == 1 {
        nc_pack_serial(arr_f, flags, dest);
        return;
    }

    let mut sums: Vec<usize> = flags
        .par_chunks(bls)
        .map(|chunk| sum_bool_serial(chunk))
        .collect();
    let m = scan_inplace(&mut sums, false, |a, b| a + b);

    *dest = Vec::with_capacity(m);
    dest.set_len(m);

    dest.par_ind_chunks_mut(&sums)
        .zip(flags.par_chunks(bls))
        .enumerate()
        .for_each(|(i, (out_chunk, flag_chunk))| {
            let s = i * bls;
            let arr_slice_f = |i, d| arr_f(s + i, d);
            nc_pack_serial_at(arr_slice_f, flag_chunk, out_chunk);
        });
}

pub unsafe fn nc_pack<T>(arr: &[T], flags: &[bool], dest: &mut Vec<T>)
where
    T: Send + Sync,
{
    if arr.len() == 0 {
        *dest = vec![];
    } else {
        let arr_f = |i, d: *mut T| std::ptr::copy(&arr[i] as *const T, d, 1);
        nc_pack_helper(arr_f, flags, dest);
    }
}

/* -------------------- Flatten -------------------- */

pub fn flatten<T>(arr: &[&Vec<T>], dest: &mut Vec<T>)
where
    T: Clone + Send + Sync + Default,
{
    let n = arr.len();
    let mut offsets: Vec<_> = (0..n).into_par_iter().map(|i| arr[i].len()).collect();
    let len = scan_inplace(&mut offsets, false, |a, b| a + b);

    *dest = maybe_uninit_vec![T::default(); len];
    dest.par_ind_chunks_mut(&offsets)
        .zip(arr.par_iter())
        .for_each(|(out_chunk, a)| {
            (*a, out_chunk)
                .into_par_iter()
                .with_gran(1024)
                .for_each(|(ai, oi)| *oi = ai.clone());
        });
}

pub fn flatten_by_val<T>(arr: &[Vec<T>], dest: &mut Vec<T>)
where
    T: Clone + Send + Sync + Default,
{
    let ref_arr: Vec<_> = arr.iter().map(|a| a).collect();
    flatten(&ref_arr, dest);
}

/* -------------------- Tokens -------------------- */

pub fn tokens<'a, T, G>(r: &'a Vec<T>, is_space: G) -> Vec<&'a [T]>
where
    T: Copy + Send + Sync + Default,
    G: Fn(&T) -> bool + Copy + Send + Sync + Sized,
{
    let to_tokens = |word: &'a [T]| word;
    return map_tokens(&r, to_tokens, is_space);
}

pub fn map_tokens<'a, T, F, G, R>(r: &'a Vec<T>, f: F, is_space: G) -> Vec<R>
where
    T: Copy + Send + Sync + Default,
    F: Fn(&'a [T]) -> R + Copy + Send + Sync + Sized,
    G: Fn(&T) -> bool + Copy + Send + Sync + Sized,
    R: Copy + Send + Sync + Default,
{
    type Ipair = (i64, i64);
    let n = r.len();

    if n == 0 {
        return vec![];
    }

    let is_start =
        |i: usize| -> bool { (i == 0 || is_space(&r[i - 1])) && (i != n) && !is_space(&r[i]) };
    let is_end =
        |i: usize| -> bool { (i == n || is_space(&r[i])) && (i != 0) && !is_space(&r[i - 1]) };

    // ipair: first = # of tokens so far, second = index of last start
    // g: given 2 ipair, a and b, check if b is a new start
    // if b is a new start, number of tokens ++, set the last seen start to b index
    let g = |a: Ipair, b: Ipair| -> Ipair {
        if b.0 == 0 {
            a
        } else {
            (a.0 + b.0, b.1)
        }
    };

    // vector storing where the token starts
    let start_tokens: Vec<Ipair> = (0..(n + 1))
        .into_par_iter()
        .with_min_len(_BLOCK_SIZE * 1000)
        .map(|i: usize| -> Ipair {
            if is_start(i) {
                (1, i as i64)
            }
            // this is a start
            else {
                (0, 0)
            } // not a start
        })
        .collect();

    // offsets.0: cumulative count of # token starts up to (and excluding) char i
    // offsets.1: last start up to (and excluding) char i
    // sum.0 = total number of tokens, sum.1: last token start
    let (offsets, sum) = block_delayed_scan(&start_tokens, g, (0, 0));

    // compute final results
    let end_indices: Vec<usize> = (0..n + 1)
        .into_par_iter()
        .filter(|index| is_end(*index))
        .map(|index| index)
        .collect();

    let results: Vec<R> = end_indices
        .into_par_iter()
        .map(|index| -> R {
            let last_start = offsets[index].1 as usize;
            f(&r[last_start..index])
        })
        .collect();

    // should have matching length
    assert!(results.len() == sum.0 as usize);

    results
}
