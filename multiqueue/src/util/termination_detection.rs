use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
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

pub struct TerminationData {
    pub idle_count: AtomicU32,
    pub no_work_count: AtomicU32,
    pub num_threads: usize,
}

impl TerminationData {
    pub fn new(num_threads: usize) -> Self {
        Self {
            idle_count: AtomicU32::new(0),
            no_work_count: AtomicU32::new(0),
            num_threads,
        }
    }
}

pub fn wait_to_terminate(data: &TerminationData) -> bool {
    let mut idle_count = data.idle_count.fetch_add(1, Relaxed) + 1;
    while idle_count < data.num_threads as u32 {
        if data.no_work_count.load(Relaxed) < data.num_threads as u32 {
            data.idle_count.fetch_sub(1, Relaxed);
            return false;
        }
        unsafe {
            core::arch::x86_64::_mm_pause();
        }
        idle_count = data.idle_count.load(Relaxed);
    }
    true
}

pub fn try_do<F, RT>(f: &F, data: &TerminationData) -> Result<RT, ()>
where
    F: Fn() -> Result<RT, ()>,
{
    if let Ok(result) = f() {
        return Ok(result);
    } else {
        let mut num_no_work = data.no_work_count.fetch_add(1, Relaxed) + 1;
        loop {
            if let Ok(result) = f() {
                data.no_work_count.fetch_sub(1, Relaxed);
                return Ok(result);
            }
            if num_no_work == data.num_threads as u32 && wait_to_terminate(data) {
                return Err(());
            }
            num_no_work = data.no_work_count.load(Relaxed);
        }
    }
}
