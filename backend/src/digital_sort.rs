use std::convert::TryFrom;

const RADIX: usize = 256;
const MASK: u64 = 0xFF;
const STR_RADIX: usize = 257;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortField {
    Id,
    Author,
    Name,
    Year,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

pub fn radix_sort_u64(data: &mut [u64]) {
    if data.len() <= 1 {
        return;
    }

    let mut buffer = vec![0_u64; data.len()];
    let mut counts = [0_usize; RADIX];

    for pass in 0..8 {
        counts.fill(0);
        let shift = pass * 8;

        for &value in data.iter() {
            let digit_u64 = (value >> shift) & MASK;
            let digit = usize::try_from(digit_u64).expect("digit fits into usize");
            counts[digit] += 1;
        }

        let mut total = 0_usize;
        for c in counts.iter_mut() {
            let current = *c;
            *c = total;
            total += current;
        }

        for &value in data.iter() {
            let digit_u64 = (value >> shift) & MASK;
            let digit = usize::try_from(digit_u64).expect("digit fits into usize");
            let pos = counts[digit];
            buffer[pos] = value;
            counts[digit] += 1;
        }

        data.copy_from_slice(&buffer);
    }
}

pub fn radix_sort_by_key<T, F>(data: &mut [T], key: F)
where
    T: Clone,
    F: Fn(&T) -> u64,
{
    radix_sort_by_key_with_direction(data, key, SortDirection::Asc);
}

pub fn radix_sort_by_key_with_direction<T, F>(data: &mut [T], key: F, direction: SortDirection)
where
    T: Clone,
    F: Fn(&T) -> u64,
{
    if data.len() <= 1 {
        return;
    }

    let mut buffer: Vec<T> = data.to_vec();
    let mut counts = [0_usize; RADIX];

    for pass in 0..8 {
        counts.fill(0);
        let shift = pass * 8;

        for item in data.iter() {
            let digit_u64 = (key(item) >> shift) & MASK;
            let digit = usize::try_from(digit_u64).expect("digit fits into usize");
            counts[digit] += 1;
        }

        let mut starts = [0_usize; RADIX];
        match direction {
            SortDirection::Asc => {
                let mut total = 0_usize;
                for i in 0..RADIX {
                    starts[i] = total;
                    total += counts[i];
                }
            }
            SortDirection::Desc => {
                let mut total = 0_usize;
                for i in (0..RADIX).rev() {
                    starts[i] = total;
                    total += counts[i];
                }
            }
        }

        for item in data.iter() {
            let digit_u64 = (key(item) >> shift) & MASK;
            let digit = usize::try_from(digit_u64).expect("digit fits into usize");
            let pos = starts[digit];
            buffer[pos] = item.clone();
            starts[digit] += 1;
        }

        data.clone_from_slice(&buffer);
    }
}

pub fn radix_sort_by_selected_field<T, FI, FA, FN, FY>(
    data: &mut [T],
    field: SortField,
    direction: SortDirection,
    id: FI,
    author: FA,
    name: FN,
    year: FY,
) where
    T: Clone,
    FI: Copy + Fn(&T) -> u64,
    FA: Copy + Fn(&T) -> &str,
    FN: Copy + Fn(&T) -> &str,
    FY: Copy + Fn(&T) -> u64,
{
    if data.len() <= 1 {
        return;
    }

    match field {
        SortField::Id => radix_sort_by_key_with_direction(data, id, direction),
        SortField::Author => radix_sort_lex_by_str_with_direction(data, author, direction),
        SortField::Name => radix_sort_lex_by_str_with_direction(data, name, direction),
        SortField::Year => radix_sort_by_key_with_direction(data, year, direction),
    }
}

fn radix_sort_lex_by_str_with_direction<T, F>(data: &mut [T], key: F, direction: SortDirection)
where
    T: Clone,
    F: Copy + Fn(&T) -> &str,
{
    let max_len = data
        .iter()
        .map(|item| key(item).as_bytes().len())
        .max()
        .unwrap_or(0);

    if max_len == 0 {
        return;
    }

    let mut buffer: Vec<T> = data.to_vec();
    let mut counts = [0_usize; STR_RADIX];

    for pos in (0..max_len).rev() {
        counts.fill(0);

        for item in data.iter() {
            let bucket = char_bucket(key(item), pos);
            counts[bucket] += 1;
        }

        let mut starts = [0_usize; STR_RADIX];
        match direction {
            SortDirection::Asc => {
                let mut total = 0_usize;
                for i in 0..STR_RADIX {
                    starts[i] = total;
                    total += counts[i];
                }
            }
            SortDirection::Desc => {
                let mut total = 0_usize;
                for i in (0..STR_RADIX).rev() {
                    starts[i] = total;
                    total += counts[i];
                }
            }
        }

        for item in data.iter() {
            let bucket = char_bucket(key(item), pos);
            let out = starts[bucket];
            buffer[out] = item.clone();
            starts[bucket] += 1;
        }

        data.clone_from_slice(&buffer);
    }
}

fn char_bucket(s: &str, pos: usize) -> usize {
    match s.as_bytes().get(pos) {
        Some(&b) => usize::from(b) + 1,
        None => 0,
    }
}

