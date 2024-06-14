use std::{collections::HashSet, hash::Hash};

pub trait ElementSet {
    type Element;
    fn with_capacity(capacity: usize) -> Self;
    fn insert(&mut self, elem: Self::Element);
    fn contains(&self, elem: &Self::Element) -> bool;
    fn remove(&mut self, elem: &Self::Element);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn iter(&self) -> impl Iterator<Item = &Self::Element>;
}

impl<T> ElementSet for HashSet<T>
where
    T: Eq + Hash,
{
    type Element = T;

    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }

    fn insert(&mut self, elem: Self::Element) {
        HashSet::insert(self, elem);
    }

    fn contains(&self, elem: &Self::Element) -> bool {
        HashSet::contains(self, elem)
    }

    fn remove(&mut self, elem: &Self::Element) {
        HashSet::remove(self, elem);
    }

    fn len(&self) -> usize {
        HashSet::len(self)
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        HashSet::iter(self)
    }
}

impl<T> ElementSet for Vec<T>
where
    T: Eq,
{
    type Element = T;

    fn with_capacity(capacity: usize) -> Self {
        Vec::with_capacity(capacity)
    }

    fn insert(&mut self, elem: Self::Element) {
        self.push(elem);
    }

    fn contains(&self, elem: &Self::Element) -> bool {
        self.as_slice().iter().any(|val| val == elem)
    }

    fn remove(&mut self, elem: &Self::Element) {
        let Some(pos) = self.as_slice().iter().position(|val| val == elem) else {
            return;
        };
        Vec::remove(self, pos);
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        self.as_slice().iter()
    }
}
