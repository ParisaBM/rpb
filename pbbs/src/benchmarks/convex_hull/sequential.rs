use crate::common::geometry::Point;

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
        while lm <= rm && !left_criterion(indices[rm]) {
            if right_criterion(indices[rm]) {
                indices[rr] = indices[rm];
                rr += 1;
            }
            rm += 1;
        }
        if lm >= rm {
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
    split(&mut indices, |i| , right_criterion);
    Vec::new()
}
