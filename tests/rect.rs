use std::ops::Deref;
use blaze_rs::{prelude::RectBox2D};

#[test]
fn col_major () {
    let rect = RectBox2D::new_col_major(&[1, 2, 3, 4, 5, 6, 7, 8, 9], 3).unwrap();
    assert_eq!(rect.as_slice(), &[1, 4, 7, 2, 5, 8, 3, 6, 9])
}

#[test]
fn transpose () {
    let rect = RectBox2D::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9], 3).unwrap();
    assert_eq!(rect.transpose().as_slice(), &[1, 4, 7, 2, 5, 8, 3, 6, 9]);
}

#[test]
fn row_iter () {
    let rect = RectBox2D::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9], 3).unwrap();
    let mut rows = rect.rows_iter();

    assert_eq!(rows.next(), Some([1, 2, 3].as_slice()));
    assert_eq!(rows.next(), Some([4, 5, 6].as_slice()));
    assert_eq!(rows.next(), Some([7, 8, 9].as_slice()));
    assert_eq!(rows.next(), None);
}

#[test]
fn col_iter () {
    let rect = RectBox2D::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9], 3).unwrap();
    let mut cols = rect.cols_iter();

    assert_eq!(cols.next().map(|x| x.copied().collect()), Some(vec![1, 4, 7]));
    assert_eq!(cols.next().map(|x| x.copied().collect()), Some(vec![2, 5, 8]));
    assert_eq!(cols.next().map(|x| x.copied().collect()), Some(vec![3, 6, 9]));
    assert_eq!(cols.next().map(|x| x.copied().collect::<Vec<_>>()), None);
}

#[test]
fn boxed () {
    let slice = [1, 2, 3, 4, 5, 6, 7, 8, 9].as_slice().to_vec().into_boxed_slice();
    let rect = RectBox2D::from_boxed_slice(slice, 3).unwrap();

    assert_eq!(rect.as_slice(), [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    let boxed = rect.into_boxed_slice();
    assert_eq!(boxed.deref(), [1, 2, 3, 4, 5, 6, 7, 8, 9].as_slice());
}

#[test]
fn zeroed () {
    let rect = RectBox2D::<i32>::new_zeroed(3, 3).unwrap();
    let rect = unsafe { rect.assume_init() };
    assert_eq!(rect.as_slice(), [0; 9].as_slice())
}