use crate::vecset::VecSet;

use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt;
use core::mem;
use core::ptr;
use core::slice;

use alloc::vec;
use alloc::vec::Vec;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct VecMap<K, V>(Vec<(K, V)>);

impl<K, V> VecMap<K, V> {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    #[must_use]
    pub fn from_single(key: K, value: V) -> Self {
        Self(vec![(key, value)])
    }

    #[inline]
    #[must_use]
    pub fn with_capacity(cap: usize) -> Self {
        Self(Vec::with_capacity(cap))
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    #[must_use]
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter(self.0.as_slice().iter())
    }

    #[inline]
    #[must_use]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut(self.0.as_mut_slice().iter_mut())
    }

    unsafe fn at_unchecked(&self, idx: usize) -> &(K, V) {
        self.0.get_unchecked(idx)
    }

    unsafe fn at_unchecked_mut(&mut self, idx: usize) -> &mut (K, V) {
        self.0.get_unchecked_mut(idx)
    }
}

impl<K: Ord, V> VecMap<K, V> {
    #[inline]
    #[must_use]
    pub fn from_vec(mut v: Vec<(K, V)>) -> Self {
        v.sort_unstable_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
        v.dedup_by(|x, first| x.0 == first.0);
        Self(v)
    }

    fn search<Q>(&self, key: &Q) -> Result<usize, usize>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.0.binary_search_by(|probe| probe.0.borrow().cmp(key))
    }

    #[inline]
    #[must_use]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.search(key).is_ok()
    }

    #[inline]
    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let idx = self.search(key).ok()?;
        let entry = unsafe { self.at_unchecked(idx) };
        Some(&entry.1)
    }

    #[inline]
    #[must_use]
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let idx = self.search(key).ok()?;
        let entry = unsafe { self.at_unchecked_mut(idx) };
        Some(&mut entry.1)
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.search(&key) {
            Ok(idx) => {
                let entry = unsafe { self.at_unchecked_mut(idx) };
                Some(mem::replace(&mut entry.1, value))
            }
            Err(idx) => {
                self.0.insert(idx, (key, value));
                None
            }
        }
    }

    #[inline]
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let idx = self.search(key).ok()?;
        let entry = self.0.remove(idx);
        Some(entry.1)
    }

    #[inline]
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        match self.search(&key) {
            Ok(idx) => Entry::Occupied(OccupiedEntry { map: self, idx }),
            Err(idx) => Entry::Vacant(VacantEntry {
                map: self,
                idx,
                key,
            }),
        }
    }

    #[inline]
    pub fn merge_copied_with(&mut self, other: &Self, mut f: impl FnMut(V, V) -> V)
    where
        K: Copy,
        V: Copy,
    {
        let lhs = &mut self.0;
        let rhs = &other.0;

        let ans_cap = lhs.len().checked_add(rhs.len()).unwrap();
        lhs.reserve(ans_cap);

        unsafe {
            let mut p1 = lhs.as_ptr();
            let mut p2 = rhs.as_ptr();
            let mut p3 = lhs.as_mut_ptr().add(lhs.len());
            let e1 = p1.add(lhs.len());
            let e2 = p2.add(rhs.len());

            while p1 < e1 && p2 < e2 {
                let (k1, v1) = &*p1;
                let (k2, v2) = &*p2;
                match Ord::cmp(k1, k2) {
                    Ordering::Less => {
                        ptr::copy_nonoverlapping(p1, p3, 1);
                        p1 = p1.add(1);
                    }
                    Ordering::Greater => {
                        ptr::copy_nonoverlapping(p2, p3, 1);
                        p2 = p2.add(1);
                    }
                    Ordering::Equal => {
                        let v = f(*v1, *v2);
                        p3.write((*k1, v));
                        p1 = p1.add(1);
                        p2 = p2.add(1);
                    }
                }
                p3 = p3.add(1);
            }
            if p1 < e1 {
                let cnt = e1.offset_from(p1) as usize;
                ptr::copy_nonoverlapping(p1, p3, cnt);
                p3 = p3.add(cnt);
            }
            if p2 < e2 {
                let cnt = e2.offset_from(p2) as usize;
                ptr::copy_nonoverlapping(p2, p3, cnt);
                p3 = p3.add(cnt);
            }
            {
                let dst = lhs.as_mut_ptr();
                let src = dst.add(lhs.len());
                let cnt = p3.offset_from(src) as usize;
                ptr::copy(src, dst, cnt);
                lhs.set_len(cnt)
            }
        }
    }

    #[inline]
    pub fn remove_less_than<Q>(&mut self, key: &Q)
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        struct Guard<'a, K, V> {
            v: &'a mut Vec<(K, V)>,
            remove_cnt: usize,
        }

        impl<K, V> Drop for Guard<'_, K, V> {
            fn drop(&mut self) {
                let v = &mut *self.v;
                let remove_cnt = self.remove_cnt;
                let remain_cnt = v.len().wrapping_sub(remove_cnt);
                unsafe {
                    let dst = v.as_mut_ptr();
                    let src = dst.add(remove_cnt);
                    ptr::copy(src, dst, remain_cnt);
                    v.set_len(remain_cnt)
                }
            }
        }

        let remove_cnt = match self.search(key) {
            Ok(idx) => idx,
            Err(idx) => idx,
        };
        if remove_cnt == 0 || remove_cnt >= self.0.len() {
            return;
        }
        let guard = Guard {
            remove_cnt,
            v: &mut self.0,
        };
        unsafe {
            let entries: *mut [(K, V)] = guard.v.get_unchecked_mut(..remove_cnt);
            ptr::drop_in_place(entries);
        }
        drop(guard);
    }

    #[inline]
    #[must_use]
    pub fn remove_max(&mut self) -> Option<(K, V)> {
        self.0.pop()
    }

    #[inline]
    pub fn apply(&self, keys: &VecSet<K>, mut f: impl FnMut(&V)) {
        unsafe {
            let mut p1 = self.0.as_ptr();
            let e1 = p1.add(self.len());
            let mut p2 = keys.as_slice().as_ptr();
            let e2 = p2.add(keys.len());

            while p1 < e1 && p2 < e2 {
                let (k1, v) = &*p1;
                let k2 = &*p2;
                match Ord::cmp(k1, k2) {
                    Ordering::Less => {
                        p1 = p1.add(1);
                    }
                    Ordering::Greater => {
                        p2 = p2.add(1);
                    }
                    Ordering::Equal => {
                        f(v);
                        p1 = p1.add(1);
                        p2 = p2.add(1);
                    }
                }
            }
        }
    }
}

impl<K: Ord, V> From<Vec<(K, V)>> for VecMap<K, V> {
    #[inline]
    fn from(v: Vec<(K, V)>) -> Self {
        Self::from_vec(v)
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for VecMap<K, V> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self::from_vec(iter.into_iter().collect())
    }
}

impl<K, V> Default for VecMap<K, V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> fmt::Debug for VecMap<K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.0.iter().map(|(k, v)| (k, v));
        f.debug_map().entries(entries).finish()
    }
}

pub struct Iter<'a, K, V>(slice::Iter<'a, (K, V)>);

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = &'a (K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl<'a, K, V> IntoIterator for &'a VecMap<K, V> {
    type Item = &'a (K, V);

    type IntoIter = Iter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct IterMut<'a, K, V>(slice::IterMut<'a, (K, V)>);

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = &'a mut (K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, K, V> IntoIterator for &'a mut VecMap<K, V> {
    type Item = &'a mut (K, V);

    type IntoIter = IterMut<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct IntoIter<K, V>(vec::IntoIter<(K, V)>);

impl<K, V> IntoIterator for VecMap<K, V> {
    type Item = (K, V);

    type IntoIter = IntoIter<K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

#[must_use]
pub enum Entry<'a, K, V>
where
    K: 'a,
    V: 'a,
{
    Vacant(VacantEntry<'a, K, V>),
    Occupied(OccupiedEntry<'a, K, V>),
}

#[must_use]
pub struct VacantEntry<'a, K, V> {
    map: &'a mut VecMap<K, V>,
    idx: usize,
    key: K,
}

#[must_use]
pub struct OccupiedEntry<'a, K, V> {
    map: &'a mut VecMap<K, V>,
    idx: usize,
}

impl<'a, K, V> Entry<'a, K, V> {
    #[inline]
    pub fn and_modify(mut self, f: impl FnOnce(&mut V)) -> Self {
        if let Entry::Occupied(ref mut e) = self {
            f(e.get_mut())
        }
        self
    }

    #[inline]
    pub fn key(&self) -> &K {
        match self {
            Entry::Vacant(e) => e.key(),
            Entry::Occupied(e) => e.key(),
        }
    }

    #[inline]
    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        self.or_insert_with(V::default)
    }

    #[inline]
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Vacant(e) => e.insert(default),
            Entry::Occupied(e) => e.into_mut(),
        }
    }

    #[inline]
    pub fn or_insert_with(self, default: impl FnOnce() -> V) -> &'a mut V {
        match self {
            Entry::Vacant(e) => e.insert(default()),
            Entry::Occupied(e) => e.into_mut(),
        }
    }

    #[inline]
    pub fn or_insert_with_key(self, default: impl FnOnce(&K) -> V) -> &'a mut V {
        match self {
            Entry::Vacant(e) => {
                let val = default(e.key());
                e.insert(val)
            }
            Entry::Occupied(e) => e.into_mut(),
        }
    }
}

impl<'a, K, V> VacantEntry<'a, K, V> {
    #[inline]
    #[must_use]
    pub fn key(&self) -> &K {
        &self.key
    }

    #[inline]
    #[must_use]
    pub fn into_key(self) -> K {
        self.key
    }

    #[inline]
    pub fn insert(self, value: V) -> &'a mut V {
        self.map.0.insert(self.idx, (self.key, value));
        let entry = unsafe { self.map.at_unchecked_mut(self.idx) };
        &mut entry.1
    }
}

impl<'a, K, V> OccupiedEntry<'a, K, V> {
    #[inline]
    #[must_use]
    pub fn get(&self) -> &V {
        let entry = unsafe { self.map.at_unchecked(self.idx) };
        &entry.1
    }

    #[inline]
    #[must_use]
    pub fn get_mut(&mut self) -> &mut V {
        let entry = unsafe { self.map.at_unchecked_mut(self.idx) };
        &mut entry.1
    }

    #[inline]
    pub fn insert(&mut self, value: V) -> V {
        mem::replace(self.get_mut(), value)
    }

    #[inline]
    #[must_use]
    pub fn into_mut(self) -> &'a mut V {
        let entry = unsafe { self.map.at_unchecked_mut(self.idx) };
        &mut entry.1
    }

    #[inline]
    #[must_use]
    pub fn key(&self) -> &K {
        let entry = unsafe { self.map.at_unchecked(self.idx) };
        &entry.0
    }

    #[inline]
    #[must_use]
    pub fn remove(self) -> V {
        self.remove_entry().1
    }

    #[inline]
    #[must_use]
    pub fn remove_entry(self) -> (K, V) {
        self.map.0.remove(self.idx)
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;

    use serde::{Deserialize, Serialize};

    impl<'de, K, V> Deserialize<'de> for VecMap<K, V>
    where
        K: Ord + Deserialize<'de>,
        V: Deserialize<'de>,
    {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<VecMap<K, V>, D::Error>
        where
            D: ::serde::de::Deserializer<'de>,
        {
            <Vec<(K, V)>>::deserialize(deserializer).map(VecMap::from_vec)
        }
    }

    impl<K, V> Serialize for VecMap<K, V>
    where
        K: Serialize,
        V: Serialize,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ::serde::ser::Serializer,
        {
            <[(K, V)]>::serialize(self.0.as_slice(), serializer)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::string::String;
    use alloc::string::ToString;

    #[test]
    fn from_vec() {
        let m: VecMap<u8, u8> =
            VecMap::from_vec(vec![(4, 1), (2, 3), (5, 7), (2, 9), (4, 6), (7, 8)]);
        assert!([1, 6].contains(m.get(&4).unwrap()));
        assert!([3, 9].contains(m.get(&2).unwrap()));
        assert_eq!(*m.get(&5).unwrap(), 7);
        assert_eq!(*m.get(&7).unwrap(), 8);
    }

    #[test]
    fn merge_max() {
        let mut m1: VecMap<u8, u8> = VecMap::from_vec(vec![(1, 1), (3, 3), (5, 5)]);
        let m2: VecMap<u8, u8> = VecMap::from_vec(vec![(1, 1), (2, 2), (3, 2), (4, 4), (5, 6)]);
        m1.merge_copied_with(&m2, |v1, v2| v1.max(v2));
        assert_eq!(*m1.get(&1).unwrap(), 1);
        assert_eq!(*m1.get(&2).unwrap(), 2);
        assert_eq!(*m1.get(&3).unwrap(), 3);
        assert_eq!(*m1.get(&4).unwrap(), 4);
        assert_eq!(*m1.get(&5).unwrap(), 6);
    }

    #[test]
    fn remove_less_than() {
        let mut m: VecMap<u8, String> = VecMap::from_vec(vec![
            (4, 1.to_string()),
            (2, 3.to_string()),
            (5, 7.to_string()),
            (2, 9.to_string()),
            (4, 6.to_string()),
            (7, 8.to_string()),
        ]);
        m.remove_less_than(&5);
        assert!(m.get(&2).is_none());
        assert!(m.get(&4).is_none());
        assert!(m.get(&5).is_some());
        assert!(m.get(&7).is_some());
    }

    #[test]
    fn apply() {
        let map = VecMap::from_iter([(1, 2), (3, 4), (5, 6)]);

        {
            let keys = VecSet::new();
            let mut ans = Vec::new();
            map.apply(&keys, |&v| ans.push(v));
            assert!(ans.is_empty());
        }
        {
            let keys = VecSet::from_single(3);
            let mut ans = Vec::new();
            map.apply(&keys, |&v| ans.push(v));
            assert_eq!(ans, [4]);
        }
        {
            let keys = VecSet::from_iter([0, 1, 2, 3, 4, 5, 6]);
            let mut ans = Vec::new();
            map.apply(&keys, |&v| ans.push(v));
            assert_eq!(ans, [2, 4, 6]);
        }
    }
}
