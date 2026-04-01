use std::cmp::Ordering;

type Link<T> = Option<Box<Node<T>>>;

#[derive(Debug)]
struct Node<T> {
    value: T,
    left: Link<T>,
    right: Link<T>,
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            left: None,
            right: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct A2Tree<T> {
    root: Link<T>,
    len: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeightedKey<T> {
    pub key: T,
    pub weight: u64,
}

impl<T> A2Tree<T> {
    pub fn new() -> Self {
        Self { root: None, len: 0 }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn insert_by<F>(&mut self, value: T, cmp: F) -> bool
    where
        F: Copy + Fn(&T, &T) -> Ordering,
    {
        let inserted = Self::insert_node(&mut self.root, value, cmp);
        if inserted {
            self.len += 1;
        }
        inserted
    }

    pub fn contains_by<F>(&self, target: &T, cmp: F) -> bool
    where
        F: Copy + Fn(&T, &T) -> Ordering,
    {
        let mut current = self.root.as_deref();
        while let Some(node) = current {
            match cmp(target, &node.value) {
                Ordering::Less => current = node.left.as_deref(),
                Ordering::Greater => current = node.right.as_deref(),
                Ordering::Equal => return true,
            }
        }
        false
    }

    pub fn inorder<'a>(&'a self, out: &mut Vec<&'a T>) {
        fn walk<'a, T>(link: &'a Link<T>, out: &mut Vec<&'a T>) {
            if let Some(node) = link {
                walk(&node.left, out);
                out.push(&node.value);
                walk(&node.right, out);
            }
        }
        walk(&self.root, out);
    }

    pub fn build_from_sorted_by_weight<F>(items: &[T], weight: F) -> Self
    where
        T: Clone,
        F: Copy + Fn(&T) -> u64,
    {
        fn build<T, F>(items: &[T], weight: F) -> Link<T>
        where
            T: Clone,
            F: Copy + Fn(&T) -> u64,
        {
            if items.is_empty() {
                return None;
            }

            let idx = a2_root_index(items, weight);
            Some(Box::new(Node {
                value: items[idx].clone(),
                left: build(&items[..idx], weight),
                right: build(&items[idx + 1..], weight),
            }))
        }

        let root = build(items, weight);
        Self {
            root,
            len: items.len(),
        }
    }

    fn insert_node<F>(node: &mut Link<T>, value: T, cmp: F) -> bool
    where
        F: Copy + Fn(&T, &T) -> Ordering,
    {
        match node {
            None => {
                *node = Some(Box::new(Node::new(value)));
                true
            }
            Some(current) => match cmp(&value, &current.value) {
                Ordering::Less => Self::insert_node(&mut current.left, value, cmp),
                Ordering::Greater => Self::insert_node(&mut current.right, value, cmp),
                Ordering::Equal => false,
            },
        }
    }
}

fn a2_root_index<T, F>(items: &[T], weight: F) -> usize
where
    F: Copy + Fn(&T) -> u64,
{
    let total_weight: u64 = items.iter().map(weight).sum();
    if total_weight == 0 {
        return items.len() / 2;
    }

    let half = total_weight / 2;
    let mut prefix = 0_u64;

    for (i, item) in items.iter().enumerate() {
        let w = weight(item);
        if prefix < half && prefix + w >= half {
            return i;
        }
        prefix += w;
    }

    items.len() - 1
}
