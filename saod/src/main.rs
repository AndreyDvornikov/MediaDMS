use project::a2_tree::A2Tree;
use project::binary_search::{binary_search_by, equal_range_by};
use project::digital_sort::{radix_sort_by_selected_field, SortDirection, SortField};

#[derive(Clone, Debug, PartialEq, Eq)]
struct SongRecord {
    id: u64,
    author: String,
    name: String,
    year: u64,
}

fn main() {
    let mut songs = vec![
        SongRecord {
            id: 12,
            author: "Muse".to_string(),
            name: "Hysteria".to_string(),
            year: 2003,
        },
        SongRecord {
            id: 8,
            author: "Arctic Monkeys".to_string(),
            name: "505".to_string(),
            year: 2007,
        },
        SongRecord {
            id: 2,
            author: "Muse".to_string(),
            name: "Hysteria".to_string(),
            year: 2003,
        },
    ];

    radix_sort_by_selected_field(
        &mut songs,
        SortField::Author,
        SortDirection::Asc,
        |x| x.id,
        |x| &x.author,
        |x| &x.name,
        |x| x.year,
    );

    let target = SongRecord {
        id: 0,
        author: "Muse".to_string(),
        name: String::new(),
        year: 0,
    };

    let idx = binary_search_by(&songs, &target, |a, b| a.author.cmp(&b.author));
    println!("binary_search_by index: {:?}", idx);

    let range = equal_range_by(&songs, &target, |a, b| a.author.cmp(&b.author));
    println!("equal_range_by: {:?}", range);

    let mut tree = A2Tree::new();
    for item in songs.iter().cloned() {
        tree.insert_by(item, |a, b| a.author.cmp(&b.author).then(a.id.cmp(&b.id)));
    }
    println!("A2 tree size: {}", tree.len());
}

