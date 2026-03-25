use std::cmp::Ordering;
use std::ops::Range;

pub fn binary_search_by<T, F>(slice: &[T], target: &T, cmp: F) -> Option<usize>
where
    F: Fn(&T, &T) -> Ordering,
{
    let idx = lower_bound_by(slice, target, &cmp);
    if idx < slice.len() && cmp(&slice[idx], target) == Ordering::Equal {
        Some(idx)
    } else {
        None
    }
}

pub fn lower_bound_by<T, F>(slice: &[T], target: &T, cmp: F) -> usize
where
    F: Fn(&T, &T) -> Ordering,
{
    let mut left = 0_usize;
    let mut right = slice.len();
    while left < right {
        let mid = left + (right - left) / 2;
        if cmp(&slice[mid], target) == Ordering::Less {
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    left
}

pub fn upper_bound_by<T, F>(slice: &[T], target: &T, cmp: F) -> usize
where
    F: Fn(&T, &T) -> Ordering,
{
    let mut left = 0_usize;
    let mut right = slice.len();
    while left < right {
        let mid = left + (right - left) / 2;
        if cmp(&slice[mid], target) == Ordering::Greater {
            right = mid;
        } else {
            left = mid + 1;
        }
    }
    left
}

pub fn equal_range_by<T, F>(slice: &[T], target: &T, cmp: F) -> Range<usize>
where
    F: Copy + Fn(&T, &T) -> Ordering,
{
    let left = lower_bound_by(slice, target, cmp);
    let right = upper_bound_by(slice, target, cmp);
    left..right
}

