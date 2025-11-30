use core::f64;

use rayon::prelude::*;

use crate::common::geometry::Point;

pub fn knn<const D: usize>(pts: &Vec<Point<D, f64>>) -> Vec<usize> {
    pts.par_iter()
        .enumerate()
        .map(|(i, pt)| {
            let (mut d, mut nearest) = (f64::INFINITY, 0);
            for (j, other_pt) in pts.iter().enumerate() {
                let new_d = (*pt - *other_pt).length_squared();
                if j != i && new_d < d {
                    nearest = j;
                    d = new_d;
                }
            }
            nearest
        })
        .collect()
}
