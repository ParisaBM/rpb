// use rayon::iter::IntoParallelRefIterator;
use crate::common::geometry::Point3d;
use rayon::prelude::*;

fn build_tree(pts: &Vec<Point3d<f64>>, effective_size: usize) {
    // let en = 

    let bounding_box = (
        pts.par_iter().cloned().reduce(
            || Point3d::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
            |a, b| Point3d::new(a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2])),
        ),
        pts.par_iter().cloned().reduce(
            || Point3d::new(-f64::INFINITY, -f64::INFINITY, -f64::INFINITY),
            |a, b| Point3d::new(a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2])),
        ),
    );

    let mut longest_dimension = 0;
    let mut width = 0.;
    for i in 0..3 {
        let dim_width = bounding_box.1[i] - bounding_box.0[i];
        if dim_width > width {
            width = dim_width;
            longest_dimension = i;
        }
    }
}

pub fn nbody(pts: &Vec<Point3d<f64>>) -> Vec<Point3d<f64>> {
    pts.clone()
}
