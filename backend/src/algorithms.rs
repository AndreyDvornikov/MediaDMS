use crate::postgres_repo::SongResponse;

// =========================
// 🔥 BINARY SEARCH
// =========================
pub fn binary_search(arr: &Vec<SongResponse>, target: i32) -> Option<usize> {
    let mut left = 0;
    let mut right = arr.len();

    while left < right {
        let mid = (left + right) / 2;

        if arr[mid].song_id == target {
            return Some(mid);
        } else if arr[mid].song_id < target {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    None
}

// =========================
// 🔥 RADIX SORT (digital)
// =========================
pub fn radix_sort(arr: &mut Vec<SongResponse>) {

    if arr.is_empty() {
        return;
    }

    let max = arr.iter().map(|x| x.song_id).max().unwrap_or(0);
    let mut exp = 1;

    while max / exp > 0 {
        counting_sort(arr, exp);
        exp *= 10;
    }
}

fn counting_sort(arr: &mut Vec<SongResponse>, exp: i32) {
    let n = arr.len();
    let mut output = vec![arr[0].clone(); n];
    let mut count = vec![0; 10];

    for i in 0..n {
        let index = (arr[i].song_id / exp % 10) as usize;
        count[index] += 1;
    }

    for i in 1..10 {
        count[i] += count[i - 1];
    }

    for i in (0..n).rev() {
        let index = (arr[i].song_id / exp % 10) as usize;
        output[count[index] - 1] = arr[i].clone();
        count[index] -= 1;
    }

    for i in 0..n {
        arr[i] = output[i].clone();
    }
}

// =========================
// 🔥 A2 TREE (упрощённое)
// =========================
pub struct A2Tree {
    data: Vec<(String, SongResponse)>
}

impl A2Tree {

    pub fn new() -> Self {
        Self { data: vec![] }
    }

    pub fn insert(&mut self, key: String, value: SongResponse) {
        self.data.push((key.to_lowercase(), value));
    }

    pub fn search(&self, key: String) -> Vec<SongResponse> {
        let k = key.to_lowercase();

        self.data.iter()
            .filter(|(kk, _)| kk.contains(&k))
            .map(|(_, v)| v.clone())
            .collect()
    }
}