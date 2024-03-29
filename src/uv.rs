extern crate nalgebra as na;
use na::{DMatrix};

#[test]
fn meantone5() {
    let mapping = DMatrix::from_vec(3, 2, vec![12, 19, 28, 7, 11, 16]);
    let mut squared = mapping.clone().insert_column(0, 0);
    squared[(0, 0)] = 1;
    let expected =
        DMatrix::from_vec(3, 3, vec![1, 0, 0, 12, 19, 28, 7, 11, 16]);
    assert_eq!(expected, squared);
    let convert_float = squared.iter().cloned().map(|x| x as f64);
    let fsquare = DMatrix::from_iterator(3, 3, convert_float);
    let inverted = fsquare.clone().try_inverse().expect("no inverse")
        * fsquare.clone().determinant();
    let uv: Vec<_> = inverted.row(0).iter().map(|&x| x as i64).collect();
    let expected = vec![-4, 4, -1];
    assert_eq!(expected, uv);
}
