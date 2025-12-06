use crate::common::geometry::{tri_area, Point};

// prefix indices, with all values from indices matching the left criterion
// and suffix with all values matching the right criterion
// there may be duplicates of these in between, or values matching neither
// the left and right criteria should be mutually exclusive
// returns the size of the prefix and suffix segment
fn split<F, G>(indices: &mut [usize], left_criterion: F, right_criterion: G) -> (usize, usize)
where
    F: Fn(usize) -> bool,
    G: Fn(usize) -> bool,
{
    let mut ll = 0;
    let mut lm = 0;
    let mut rr = indices.len() - 1;
    let mut rm = indices.len() - 1;
    loop {
        while lm <= rm && !right_criterion(indices[lm]) {
            if left_criterion(indices[lm]) {
                indices[ll] = indices[lm];
                ll += 1;
            }
            lm += 1;
        }
        while lm.cast_signed() <= rm.cast_signed() && !left_criterion(indices[rm]) {
            if right_criterion(indices[rm]) {
                indices[rr] = indices[rm];
                rr -= 1;
            }
            // special handling if the whole slice needs to go into the right segment
            rm = rm.wrapping_sub(1);
        }
        if lm.cast_signed() >= rm.cast_signed() {
            break;
        }
        let temp = indices[lm];
        indices[ll] = indices[rm];
        indices[rr] = temp;
        lm += 1;
        ll += 1;
        rr -= 1;
        rm -= 1;
    }
    (ll, indices.len() - rr - 1)
}

fn serial_quick_hull(
    indices: &mut [usize],
    pts: &Vec<Point<2, f64>>,
    leftmost: usize,
    rightmost: usize,
) -> usize {
    if indices.len() < 2 {
        return indices.len();
    }
    let max_area = *indices
        .iter()
        .map(|i| (i, tri_area(pts[leftmost], pts[rightmost], pts[*i])))
        .max_by(|(_, a0), (_, a1)| a0.total_cmp(a1))
        .unwrap()
        .0;

    let (left_length, right_length) = split(
        indices,
        |i| tri_area(pts[leftmost], pts[max_area], pts[i]) > 0.0,
        |i| tri_area(pts[max_area], pts[rightmost], pts[i]) > 0.0,
    );
    let mid_length = indices.len() - left_length - right_length;

    let (left_segment, mut right_segment) = indices.split_at_mut(left_length);
    right_segment = &mut right_segment[mid_length..];

    let left_solution = serial_quick_hull(left_segment, pts, leftmost, max_area);
    let right_solution = serial_quick_hull(right_segment, pts, max_area, rightmost);

    indices[left_solution] = max_area;
    for i in 0..right_solution {
        indices[left_solution + 1 + i] = indices[indices.len() - right_length + i];
    }

    left_solution + 1 + right_solution
}

pub fn hull(pts: &Vec<Point<2, f64>>) -> Vec<usize> {
    let rightmost = pts
        .iter()
        .enumerate()
        .max_by(|(_, p), (_, q)| p[0].total_cmp(&q[0]))
        .unwrap()
        .0;
    let leftmost = pts
        .iter()
        .enumerate()
        .min_by(|(_, p), (_, q)| p[0].total_cmp(&q[0]).then_with(|| p[1].total_cmp(&q[1])))
        .unwrap()
        .0;

    let mut indices = (0..pts.len()).collect::<Vec<_>>();
    let (above_length, below_length) = split(
        &mut indices,
        |i| tri_area(pts[leftmost], pts[rightmost], pts[i]) > 0.0,
        |i| tri_area(pts[rightmost], pts[leftmost], pts[i]) > 0.0,
    );
    let mid_length = pts.len() - above_length - below_length;

    let (above_segment, mut below_segment) = indices.split_at_mut(above_length);
    below_segment = &mut below_segment[mid_length..];

    let top_solution = serial_quick_hull(above_segment, pts, leftmost, rightmost);
    let bottom_solution = serial_quick_hull(below_segment, pts, rightmost, leftmost);

    let mut result = Vec::with_capacity(top_solution + bottom_solution + 2);
    result.push(leftmost);
    result.extend_from_slice(&indices[..top_solution]);
    result.push(rightmost);
    result.extend_from_slice(&indices[indices.len() - below_length..indices.len() - below_length + bottom_solution]);

    result
}
