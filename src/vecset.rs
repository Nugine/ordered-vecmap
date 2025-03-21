use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt;
use core::mem;
use core::ptr;
use core::slice;

use alloc::vec;
use alloc::vec::Vec;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct VecSet<T>(Vec<T>);

impl<T> VecSet<T> {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    #[must_use]
    pub fn from_single(val: T) -> Self {
        Self(vec![val])
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
    pub fn as_slice(&self) -> &[T] {
        self.0.as_slice()
    }

    #[inline]
    #[must_use]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter(self.0.as_slice().iter())
    }

    #[inline]
    #[must_use]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut(self.0.as_mut_slice().iter_mut())
    }
}

impl<T: Ord> VecSet<T> {
    #[inline]
    #[must_use]
    pub fn from_vec(mut v: Vec<T>) -> Self {
        v.sort_unstable();
        v.dedup_by(|x, first| x == first);
        Self(v)
    }

    fn search<Q>(&self, val: &Q) -> Result<usize, usize>
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.0.binary_search_by(|probe| probe.borrow().cmp(val))
    }

    #[inline]
    #[must_use]
    pub fn contains<Q>(&self, val: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.search(val).is_ok()
    }

    #[inline]
    #[must_use]
    pub fn insert(&mut self, val: T) -> Option<T> {
        match self.search(&val) {
            Ok(idx) => {
                let prev = unsafe { &mut self.0.get_unchecked_mut(idx) };
                Some(mem::replace(prev, val))
            }
            Err(idx) => {
                self.0.insert(idx, val);
                None
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn remove<Q>(&mut self, val: &Q) -> Option<T>
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        match self.search(val) {
            Ok(idx) => Some(self.0.remove(idx)),
            Err(_) => None,
        }
    }

    #[inline]
    pub fn union_copied_inplace(&mut self, other: &Self)
    where
        T: Copy,
    {
        let lhs = &mut self.0;
        let rhs = &other.0;

        let ans_cap = lhs.len().checked_add(rhs.len()).unwrap();
        lhs.reserve(ans_cap);

        unsafe {
            let p1 = lhs.as_ptr();
            let p2 = rhs.as_ptr();
            let p3 = lhs.as_mut_ptr().add(lhs.len());
            let e1 = p1.add(lhs.len());
            let e2 = p2.add(rhs.len());

            let end = raw_union_copied(p1, p2, p3, e1, e2);

            let dst = lhs.as_mut_ptr();
            let src = dst.add(lhs.len());
            let cnt = end.offset_from(src) as usize;
            ptr::copy(src, dst, cnt);
            lhs.set_len(cnt)
        }
    }

    #[inline]
    #[must_use]
    pub fn union_copied(&self, other: &Self) -> Self
    where
        T: Copy,
    {
        let lhs = &self.0;
        let rhs = &other.0;

        let ans_cap = lhs.len().checked_add(rhs.len()).unwrap();
        let mut ans = Vec::with_capacity(ans_cap);

        unsafe {
            let p1 = lhs.as_ptr();
            let p2 = rhs.as_ptr();
            let p3 = ans.as_mut_ptr();
            let e1 = p1.add(lhs.len());
            let e2 = p2.add(rhs.len());

            let end = raw_union_copied(p1, p2, p3, e1, e2);
            let cnt = end.offset_from(p3) as usize;
            ans.set_len(cnt);
        }

        Self(ans)
    }

    #[inline]
    #[must_use]
    pub fn intersection_copied(&self, other: &Self) -> Self
    where
        T: Copy,
    {
        let lhs = &self.0;
        let rhs = &other.0;

        let ans_cap = lhs.len().min(rhs.len());
        let mut ans = Vec::with_capacity(ans_cap);

        unsafe {
            let p1 = lhs.as_ptr();
            let p2 = rhs.as_ptr();
            let p3 = ans.as_mut_ptr();
            let e1 = p1.add(lhs.len());
            let e2 = p2.add(rhs.len());

            let end = raw_intersection_copied(p1, p2, p3, e1, e2);
            let cnt = end.offset_from(p3) as usize;
            ans.set_len(cnt)
        }

        Self(ans)
    }

    #[inline]
    pub fn difference_copied_inplace(&mut self, other: &Self)
    where
        T: Copy,
    {
        let lhs = &mut self.0;
        let rhs = &other.0;

        let ans_cap = lhs.len();
        lhs.reserve(ans_cap);

        unsafe {
            let p1 = lhs.as_ptr();
            let p2 = rhs.as_ptr();
            let p3 = lhs.as_mut_ptr().add(lhs.len());
            let e1 = p1.add(lhs.len());
            let e2 = p2.add(rhs.len());

            let end = raw_difference_copied(p1, p2, p3, e1, e2);

            let dst = lhs.as_mut_ptr();
            let src = dst.add(lhs.len());
            let cnt = end.offset_from(src) as usize;
            ptr::copy(src, dst, cnt);
            lhs.set_len(cnt)
        }
    }
}

impl<T: Ord> From<Vec<T>> for VecSet<T> {
    #[inline]
    fn from(v: Vec<T>) -> Self {
        Self::from_vec(v)
    }
}

impl<T: Ord> FromIterator<T> for VecSet<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_vec(iter.into_iter().collect())
    }
}

impl<T> Default for VecSet<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: fmt::Debug> fmt::Debug for VecSet<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.0.iter()).finish()
    }
}

pub struct Iter<'a, T>(slice::Iter<'a, T>);

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, T> IntoIterator for &'a VecSet<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct IterMut<'a, T>(slice::IterMut<'a, T>);

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, T> IntoIterator for &'a mut VecSet<T> {
    type Item = &'a mut T;

    type IntoIter = IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct IntoIter<T>(vec::IntoIter<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<T> IntoIterator for VecSet<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}

unsafe fn raw_union_copied<T: Copy + Ord>(
    mut p1: *const T,
    mut p2: *const T,
    mut p3: *mut T,
    e1: *const T,
    e2: *const T,
) -> *mut T {
    while p1 < e1 && p2 < e2 {
        match Ord::cmp(&*p1, &*p2) {
            Ordering::Less => {
                ptr::copy_nonoverlapping(p1, p3, 1);
                p1 = p1.add(1);
            }
            Ordering::Greater => {
                ptr::copy_nonoverlapping(p2, p3, 1);
                p2 = p2.add(1);
            }
            Ordering::Equal => {
                ptr::copy_nonoverlapping(p1, p3, 1);
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
    p3
}

unsafe fn raw_intersection_copied<T: Copy + Ord>(
    mut p1: *const T,
    mut p2: *const T,
    mut p3: *mut T,
    e1: *const T,
    e2: *const T,
) -> *mut T {
    while p1 < e1 && p2 < e2 {
        match Ord::cmp(&*p1, &*p2) {
            Ordering::Less => {
                p1 = p1.add(1);
            }
            Ordering::Greater => {
                p2 = p2.add(1);
            }
            Ordering::Equal => {
                ptr::copy_nonoverlapping(p1, p3, 1);
                p1 = p1.add(1);
                p2 = p2.add(1);
                p3 = p3.add(1);
            }
        }
    }
    p3
}

unsafe fn raw_difference_copied<T: Copy + Ord>(
    mut p1: *const T,
    mut p2: *const T,
    mut p3: *mut T,
    e1: *const T,
    e2: *const T,
) -> *mut T {
    while p1 < e1 && p2 < e2 {
        match Ord::cmp(&*p1, &*p2) {
            Ordering::Less => {
                ptr::copy_nonoverlapping(p1, p3, 1);
                p1 = p1.add(1);
                p3 = p3.add(1);
            }
            Ordering::Greater => {
                p2 = p2.add(1);
            }
            Ordering::Equal => {
                p1 = p1.add(1);
                p2 = p2.add(1);
            }
        }
    }
    if p1 < e1 {
        let cnt = e1.offset_from(p1) as usize;
        ptr::copy_nonoverlapping(p1, p3, cnt);
        p3 = p3.add(cnt);
    }
    p3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_vec() {
        let s = VecSet::<u64>::from_vec(vec![1, 4, 3, 2, 5, 7, 9, 2, 4, 6, 7, 8, 0]);
        assert_eq!(s.as_slice(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
    }

    #[test]
    fn union() {
        {
            let mut s1 = VecSet::<u64>::from_iter([1, 2, 3, 5]);
            let s2 = VecSet::<u64>::from_iter([2, 4, 5, 6]);
            s1.union_copied_inplace(&s2);
            assert_eq!(s1.as_slice(), &[1, 2, 3, 4, 5, 6])
        }
        {
            let s1 = VecSet::<u64>::from_iter([1, 2, 3, 5]);
            let s2 = VecSet::<u64>::from_iter([2, 4, 5, 6]);
            let s3 = s1.union_copied(&s2);
            assert_eq!(s3.as_slice(), &[1, 2, 3, 4, 5, 6])
        }
    }

    #[test]
    fn intersection() {
        let s1 = VecSet::<u64>::from_vec(vec![1, 2, 3, 5]);
        let s2 = VecSet::<u64>::from_vec(vec![2, 4, 5, 6]);
        let s3 = s1.intersection_copied(&s2);
        assert_eq!(s3.as_slice(), &[2, 5])
    }

    #[test]
    fn difference() {
        {
            let mut s1 = VecSet::<u64>::from_iter([1, 2, 3, 5]);
            let s2 = VecSet::<u64>::from_iter([2, 4, 5, 6]);
            s1.difference_copied_inplace(&s2);
            assert_eq!(s1.as_slice(), &[1, 3])
        }
        {
            let mut s1 = VecSet::<u64>::from_iter([1, 2, 3, 5]);
            let s2 = VecSet::<u64>::from_iter([]);
            s1.difference_copied_inplace(&s2);
            assert_eq!(s1.as_slice(), &[1, 2, 3, 5])
        }
        {
            let mut s1 = VecSet::<u64>::from_iter([3]);
            let s2 = VecSet::<u64>::from_iter([1, 2, 4, 5]);
            s1.difference_copied_inplace(&s2);
            assert_eq!(s1.as_slice(), &[3])
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;

    use serde::{Deserialize, Serialize};

    impl<'de, T: Ord + Deserialize<'de>> Deserialize<'de> for VecSet<T> {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<VecSet<T>, D::Error>
        where
            D: ::serde::de::Deserializer<'de>,
        {
            <Vec<T>>::deserialize(deserializer).map(VecSet::from_vec)
        }
    }

    impl<T: Serialize> Serialize for VecSet<T> {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ::serde::ser::Serializer,
        {
            <[T]>::serialize(self.as_slice(), serializer)
        }
    }
}
