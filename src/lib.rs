#![deny(
    clippy::all,
    clippy::cargo,
    clippy::indexing_slicing,
    clippy::must_use_candidate
)]

mod iter;
use self::iter::Iter;

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::mem;

pub struct OrderedVecMap<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
}

impl<K, V> OrderedVecMap<K, V> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
        }
    }

    #[must_use]
    pub fn from_vec(mut kv: Vec<(K, V)>) -> Self
    where
        K: Ord,
    {
        kv.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
        kv.dedup_by(|x, first| x.0 == first.0);

        let mut keys = Vec::with_capacity(kv.len());
        let mut values = Vec::with_capacity(kv.len());

        for (k, v) in kv {
            keys.push(k);
            values.push(v);
        }

        Self { keys, values }
    }

    #[must_use]
    pub fn keys_slice(&self) -> &[K] {
        self.keys.as_slice()
    }

    #[must_use]
    pub fn values_slice(&self) -> &[V] {
        self.values.as_slice()
    }

    fn search<Q>(&self, key: &Q) -> Result<usize, usize>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        let ks = self.keys_slice();
        ks.binary_search_by(|probe| probe.borrow().cmp(key))
    }

    /// Performs a binary search
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.search(key).is_ok()
    }

    /// Performs a binary search
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        debug_assert_eq!(self.keys.len(), self.values.len());
        let index = self.search(key).ok()?;
        Some(unsafe { self.values.get_unchecked(index) })
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        debug_assert!(self.keys.len() == self.values.len());
        let index = self.search(key).ok()?;
        Some(unsafe { self.values.get_unchecked_mut(index) })
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        debug_assert_eq!(self.keys.len(), self.values.len());
        let index = self.search(key).ok()?;
        self.keys.remove(index);
        Some(self.values.remove(index))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    #[must_use]
    pub fn iter(&self) -> Iter<'_, K, V> {
        debug_assert_eq!(self.keys.len(), self.values.len());
        Iter::new(self)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        enum Position {
            Equal(usize),
            Insert(usize),
            End,
        }

        debug_assert_eq!(self.keys.len(), self.values.len());

        let order = match self.keys_slice().last() {
            None => Ordering::Greater,
            Some(max_key) => key.cmp(max_key),
        };

        let pos = match order {
            Ordering::Less => match self.search(&key) {
                Ok(index) => Position::Equal(index),
                Err(index) => Position::Insert(index),
            },
            Ordering::Equal => Position::Equal(self.keys.len().wrapping_sub(1)),
            Ordering::Greater => Position::End,
        };

        if !matches!(pos, Position::Equal(_)) {
            self.keys.reserve(1);
            self.values.reserve(1);
        }

        match pos {
            Position::Equal(index) => {
                let v = unsafe { self.values.get_unchecked_mut(index) };
                Some(mem::replace(v, value))
            }
            Position::Insert(index) => {
                self.keys.insert(index, key);
                self.values.insert(index, value);
                None
            }
            Position::End => {
                self.keys.push(key);
                self.values.push(value);
                None
            }
        }
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for OrderedVecMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self::from_vec(iter.into_iter().collect())
    }
}

impl<'a, K, V> IntoIterator for &'a OrderedVecMap<K, V> {
    type Item = (&'a K, &'a V);

    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        OrderedVecMap::iter(self)
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    use std::cmp::Ordering;
    use std::ops::Not;

    #[test]
    fn simple() {
        let n: i32 = 100;
        let map = (0..n).map(|x| (x, x)).collect::<OrderedVecMap<i32, i32>>();
        for i in (-n)..(n * 2) {
            if (0..n).contains(&i) {
                assert!(map.contains_key(&i));
                assert_eq!(map.get(&i), Some(&i));
            } else {
                assert!(map.contains_key(&i).not());
                assert!(map.get(&i).is_none());
            }
        }
        for (x, y) in &map {
            assert_eq!(x, y);
        }
    }

    #[test]
    fn strings() {
        let n: i32 = 100;
        let mut map = OrderedVecMap::new();
        for x in 0..n {
            let s1 = x.to_string();
            let s2 = s1.clone();
            assert!(map.insert(s1, s2).is_none());
        }

        for i in (-n)..(n * 2) {
            let s = i.to_string();
            if (0..n).contains(&i) {
                assert!(map.contains_key(s.as_str()));
                assert_eq!(map.get(s.as_str()), Some(&s));
                assert_eq!(map.remove(&s).as_deref(), Some(s.as_str()));
            } else {
                assert!(map.contains_key(&s).not());
                assert!(map.get(&s).is_none());
            }
        }
    }

    #[test]
    fn weird() {
        #[derive(Debug)]
        struct RandomOrder<T>(T);

        fn random_order() -> Ordering {
            let mut rng = rand::thread_rng();
            let x: i8 = rng.gen_range(-1..2);
            match x {
                -1 => Ordering::Less,
                0 => Ordering::Equal,
                1 => Ordering::Greater,
                _ => unreachable!(),
            }
        }

        impl<T> PartialEq for RandomOrder<T> {
            fn eq(&self, _: &Self) -> bool {
                random_order() == Ordering::Equal
            }
        }
        impl<T> Eq for RandomOrder<T> {}

        impl<T> PartialOrd for RandomOrder<T> {
            fn partial_cmp(&self, _: &Self) -> Option<Ordering> {
                Some(random_order())
            }
        }

        impl<T> Ord for RandomOrder<T> {
            fn cmp(&self, _: &Self) -> Ordering {
                random_order()
            }
        }

        impl<T> Borrow<T> for RandomOrder<T> {
            fn borrow(&self) -> &T {
                &self.0
            }
        }

        let n: i32 = 100;

        let mut map = OrderedVecMap::new();

        for x in 0..n {
            let _ = map.insert(RandomOrder(x), x);
        }

        // dbg!(map.len());

        for i in (-n)..(n * 2) {
            let _ = map.contains_key(&i);
            let _ = map.get(&i);
            let _ = map.remove(&i);
        }
    }
}
