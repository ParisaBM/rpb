use rayon::prelude::*;
use std::cmp::Ordering;

use crate::common::geometry::Point;

const KEY_BITS: usize = 60;

const fn max_val(d: usize) -> u64 {
    (1 << (KEY_BITS / d) as usize) - 1
}

// converts float to integer representation of points
// add the index to each point because they're going to be rearranged
fn convert<const D: usize>(pts: &Vec<Point<D, f64>>) -> Vec<(usize, Point<D, u64>)> {
    let bounding_box = (
        pts.par_iter().cloned().reduce(
            || Point::new([f64::INFINITY; D]),
            |a, b| Point::new(std::array::from_fn(|i| a[i].min(b[i]))),
        ),
        pts.par_iter().cloned().reduce(
            || Point::new([-f64::INFINITY; D]),
            |a, b| Point::new(std::array::from_fn(|i| a[i].max(b[i]))),
        ),
    );
    let delta = (0..D)
        .map(|i| bounding_box.1[i] - bounding_box.0[i])
        .max_by(|x, y| x.total_cmp(y))
        .unwrap();

    pts.par_iter()
        .map(|p| {
            Point::new(std::array::from_fn(|i| {
                ((p[i] - bounding_box.0[i]) / delta * max_val(D) as f64) as u64
            }))
        })
        .enumerate()
        .collect()
}

// returns true if the most significant bit of x is lower than that of y
fn less_msb(x: u64, y: u64) -> bool {
    x < y && x < x ^ y
}

// compares the interleaved bit of p and q
// for example if p had 3 bits in each dimension we could write: p = x0 x1 x2 y0 y1 y2 z0 z1 z2
// interleaving would produce instead: x0 y0 z0 x1 y1 z1 x2 y2 z2
// does not explicitly rearange the bits
fn cmp_shuffle<const D: usize>(p: Point<D, u64>, q: Point<D, u64>) -> Ordering {
    let mut highest_difference_bit = 0;
    let mut highest_difference_d = 0;
    for i in 0..D {
        let difference = p[i] ^ q[i];
        if less_msb(highest_difference_bit, difference) {
            highest_difference_bit = difference;
            highest_difference_d = i;
        }
    }
    p[highest_difference_d].cmp(&q[highest_difference_d])
}

// sorts by interleaved bits, creating a flattened octree
fn preprocess<const D: usize>(pts: &mut Vec<(usize, Point<D, u64>)>) {
    pts.par_sort_by(|p, q| cmp_shuffle(p.1, q.1));
}

// returns a number where after the first 1 bit, all subsequent bits are set to 1
// i.e. 00010011 would become 00011111
fn set_below_msb(x: u64) -> u64 {
    let x = x | x >> 1;
    let x = x | x >> 2;
    let x = x | x >> 4;
    let x = x | x >> 8;
    x | x >> 16
}

// finds the smallest possible distance between a point and a box
// the points aren't necessarily the corners of the box,
// rather they're the first and last in a list of points sorted using the above method
// thus we check how many leading bits are the same in every dimension at both endpoints,
// and extrapolate the largest possible box based on that information
fn dist_sq_to_box<const D: usize>(
    p: Point<D, u64>,
    bounding_box: (Point<D, u64>, Point<D, u64>),
) -> u64 {
    let mut highest_difference = 0;
    for i in 0..D {
        let difference = bounding_box.0[i] ^ bounding_box.1[i];
        if less_msb(highest_difference, difference) {
            highest_difference = difference;
        }
    }
    let box_size = set_below_msb(highest_difference);

    let mut distance = 0;
    for i in 0..D {
        let lower_bound = bounding_box.0[i] & !box_size;
        let upper_bound = lower_bound + box_size;
        if p[i] < lower_bound {
            distance += (lower_bound - p[i]).pow(2);
        } else if p[i] > upper_bound {
            distance += (p[i] - upper_bound).pow(2);
        }
    }

    distance
}

struct ChanNn<const D: usize> {
    ans: usize,
    least_dist_squared: u64,
    // where points could possibly fall
    bounding_box: (Point<D, u64>, Point<D, u64>),
}

impl<const D: usize> ChanNn<D> {
    fn check_dist(&mut self, p: Point<D, u64>, q: Point<D, u64>) -> bool {
        if p == q {
            return false;
        }
        let dist_squared = (p - q).length_squared();

        if dist_squared > self.least_dist_squared {
            return false;
        }

        self.least_dist_squared = dist_squared;
        let dist = (dist_squared as f64).sqrt().ceil() as u64;

        for i in 0..D {
            self.bounding_box.0[i] = q[i].saturating_sub(dist);
            self.bounding_box.1[i] = (q[i] + dist).min(max_val(D));
        }
        true
    }

    fn sss_query_0(
        &mut self,
        pts: &Vec<(usize, Point<D, u64>)>,
        bounds: (usize, usize),
        p: Point<D, u64>,
    ) {
        if bounds.0 == bounds.1 {
            return;
        }
        let mid = (bounds.0 + bounds.1) / 2;
        if self.check_dist(p, pts[mid].1) {
            self.ans = mid;
        }
        if pts.len() == 1
            || dist_sq_to_box(p, (pts[bounds.0].1, pts[bounds.1 - 1].1)) >= self.least_dist_squared
        {
            return;
        }

        if cmp_shuffle(p, pts[mid].1) == Ordering::Less {
            self.sss_query_0(pts, (bounds.0, mid), p);
            if cmp_shuffle(self.bounding_box.1, pts[mid].1) == Ordering::Greater {
                self.sss_query_0(pts, (mid + 1, bounds.1), p);
            }
        } else {
            self.sss_query_0(pts, (mid + 1, bounds.1), p);
            if cmp_shuffle(self.bounding_box.0, pts[mid].1) == Ordering::Less {
                self.sss_query_0(pts, (bounds.0, mid), p);
            }
        }
    }

    fn sss_query(pts: &Vec<(usize, Point<D, u64>)>, p: Point<D, u64>) -> usize {
        let mut chan = ChanNn {
            ans: 0,
            least_dist_squared: u64::MAX,
            bounding_box: (Point::new([0; D]), Point::new([max_val(D); D])),
        };
        chan.sss_query_0(pts, (0, pts.len()), p);
        pts[chan.ans].0
    }
}

pub fn knn<const D: usize>(pts: &Vec<Point<D, f64>>) -> Vec<usize> {
    let mut pts = convert(pts);
    preprocess(&mut pts);

    let mut result = vec![0; pts.len()];
    pts.par_iter()
        .map(|p| (p.0, ChanNn::sss_query(&pts, p.1)))
        .collect::<Vec<_>>()
        .iter()
        .for_each(|(i, j)| {
            result[*i] = *j;
        });
    result
}
