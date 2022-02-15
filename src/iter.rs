use crate::OrderedVecMap;

use std::iter::FusedIterator;
use std::slice;

pub struct Iter<'a, K, V> {
    keys_iter: slice::Iter<'a, K>,
    values_iter: slice::Iter<'a, V>,
}

impl<'a, K, V> Iter<'a, K, V> {
    pub(crate) fn new(map: &'a OrderedVecMap<K, V>) -> Self {
        Self {
            keys_iter: map.keys_slice().iter(),
            values_iter: map.values_slice().iter(),
        }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let key = self.keys_iter.next()?;
        let value = self.values_iter.next().unwrap();
        Some((key, value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.keys_iter.size_hint()
    }
}

impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V> {}
impl<'a, K, V> FusedIterator for Iter<'a, K, V> {}

impl<'a, K, V> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let key = self.keys_iter.next_back()?;
        let value = self.values_iter.next_back().unwrap();
        Some((key, value))
    }
}
