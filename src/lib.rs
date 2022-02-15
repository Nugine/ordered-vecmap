#![deny(clippy::all, clippy::cargo)]

mod iter;
use self::iter::Iter;

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::mem;
use std::mem::ManuallyDrop;
use std::ptr;
use std::ptr::NonNull;
use std::slice;

/// An ordered map based on vectors and binary search.
pub struct OrderedVecMap<K, V> {
    keys: NonNull<K>,
    values: NonNull<V>,
    length: usize,
    capacity: usize,
    _marker: PhantomData<(Vec<K>, Vec<V>)>,
}

unsafe impl<K: Send, V: Send> Send for OrderedVecMap<K, V> {}
unsafe impl<K: Sync, V: Sync> Sync for OrderedVecMap<K, V> {}

impl<K, V> OrderedVecMap<K, V> {
    unsafe fn compress(keys_vec: Vec<K>, values_vec: Vec<V>) -> Self {
        let mut keys_vec = ManuallyDrop::new(keys_vec);
        let mut values_vec = ManuallyDrop::new(values_vec);

        let length = keys_vec.len();
        let capacity = keys_vec.capacity();

        let keys = NonNull::new_unchecked(keys_vec.as_mut_ptr());
        let values = NonNull::new_unchecked(values_vec.as_mut_ptr());

        Self {
            keys,
            values,
            length,
            capacity,
            _marker: PhantomData,
        }
    }

    unsafe fn uncompress(&mut self) -> (Vec<K>, Vec<V>) {
        let keys = self.keys.as_ptr();
        let values = self.values.as_ptr();
        let len = self.length;
        let cap = self.capacity;
        let keys_vec = Vec::from_raw_parts(keys, len, cap);
        let values_vec = Vec::from_raw_parts(values, len, cap);
        (keys_vec, values_vec)
    }

    pub fn new() -> Self {
        unsafe { Self::compress(Vec::new(), Vec::new()) }
    }

    pub fn from_vec(mut vec: Vec<(K, V)>) -> Self
    where
        K: Ord,
    {
        vec.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
        vec.dedup_by(|lhs, rhs| lhs.0.eq(&rhs.0));

        let length = vec.len();
        let mut keys_vec = Vec::with_capacity(length);
        let mut values_vec = Vec::with_capacity(length);

        for (k, v) in vec {
            keys_vec.push(k);
            values_vec.push(v);
        }

        unsafe { Self::compress(keys_vec, values_vec) }
    }

    pub fn keys_slice(&self) -> &[K] {
        unsafe { slice::from_raw_parts(self.keys.as_ptr(), self.length) }
    }

    pub fn values_slice(&self) -> &[V] {
        unsafe { slice::from_raw_parts(self.values.as_ptr(), self.length) }
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
        match self.search(key) {
            Ok(index) => Some(unsafe { array_get(self.values, index) }),
            Err(_) => None,
        }
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        match self.search(key) {
            Ok(index) => Some(unsafe { array_get_mut(self.values, index) }),
            Err(_) => None,
        }
    }

    unsafe fn mutate<R>(&mut self, f: impl FnOnce(&mut Vec<K>, &mut Vec<V>) -> R) -> R {
        let (mut keys_vec, mut values_vec) = {
            let empty = Self::new();
            let vecs = self.uncompress();
            ptr::write(self, empty);
            vecs
        };
        let ans = f(&mut keys_vec, &mut values_vec);
        let this = Self::compress(keys_vec, values_vec);
        ptr::write(self, this);
        ans
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        match self.search(&key) {
            Ok(index) => {
                let v = unsafe { array_get_mut(self.values, index) };
                Some(mem::replace(v, value))
            }
            Err(index) => {
                unsafe {
                    self.mutate(|ks, vs| {
                        ks.insert(index, key);
                        vs.insert(index, value);
                    })
                };
                None
            }
        }
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        match self.search(key) {
            Ok(index) => Some(unsafe {
                self.mutate(|ks, vs| {
                    ks.remove(index);
                    vs.remove(index)
                })
            }),
            Err(_) => None,
        }
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter::new(self)
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn insert_max(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        let order = match self.keys_slice() {
            [] => Ordering::Greater,
            [.., max_key] => key.cmp(max_key),
        };

        match order {
            Ordering::Less => self.insert(key, value),
            Ordering::Equal => {
                let index = self.length.wrapping_sub(1);
                let v = unsafe { array_get_mut(self.values, index) };
                Some(mem::replace(v, value))
            }
            Ordering::Greater => {
                unsafe {
                    self.mutate(|ks, vs| {
                        ks.push(key);
                        vs.push(value);
                    });
                }
                None
            }
        }
    }
}

#[inline(always)]
unsafe fn array_get<'a, T>(base: NonNull<T>, index: usize) -> &'a T {
    &*base.as_ptr().add(index)
}

#[inline(always)]
unsafe fn array_get_mut<'a, T>(base: NonNull<T>, index: usize) -> &'a mut T {
    &mut *base.as_ptr().add(index)
}

impl<K, V> Drop for OrderedVecMap<K, V> {
    fn drop(&mut self) {
        unsafe { drop(self.uncompress()) }
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
        self.iter()
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
            let _ = map.insert_max(RandomOrder(x), x);
        }

        // dbg!(map.len());

        for i in (-n)..(n * 2) {
            let _ = map.contains_key(&i);
            let _ = map.get(&i);
            let _ = map.remove(&i);
        }
    }
}
