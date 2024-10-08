use serde::Deserialize;
use std::mem;
use std::slice::Iter;
use std::vec::IntoIter;

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> From<OneOrMany<T>> for Vec<T> {
    fn from(value: OneOrMany<T>) -> Self {
        match value {
            OneOrMany::One(t) => vec![t],
            OneOrMany::Many(vec) => vec,
        }
    }
}

impl<T> IntoIterator for OneOrMany<T> {
    type Item = T;
    type IntoIter = OneOrManyIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            OneOrMany::One(t) => OneOrManyIter::One(Some(t)),
            OneOrMany::Many(vec) => OneOrManyIter::Many(vec.into_iter()),
        }
    }
}

pub enum OneOrManyIter<T> {
    One(Option<T>),
    Many(IntoIter<T>),
}

impl<T> Iterator for OneOrManyIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OneOrManyIter::One(t) => mem::take(t),
            OneOrManyIter::Many(vec) => vec.next(),
        }
    }
}

impl<'a, T> IntoIterator for &'a OneOrMany<T> {
    type Item = &'a T;
    type IntoIter = OneOrManyIterRef<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            OneOrMany::One(t) => OneOrManyIterRef::One(Some(t)),
            OneOrMany::Many(vec) => OneOrManyIterRef::Many(vec.iter()),
        }
    }
}

#[derive(Clone)]
pub enum OneOrManyIterRef<'a, T> {
    One(Option<&'a T>),
    Many(Iter<'a, T>),
}

impl<'a, T> Iterator for OneOrManyIterRef<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OneOrManyIterRef::One(t) => mem::take(t),
            OneOrManyIterRef::Many(vec) => vec.next(),
        }
    }
}

impl<T> OneOrMany<T> {
    pub(crate) fn len(&self) -> usize {
        match self {
            OneOrMany::One(_) => 1,
            OneOrMany::Many(vec) => vec.len(),
        }
    }

    pub(crate) fn iter(&self) -> OneOrManyIterRef<T> {
        self.into_iter()
    }
}
