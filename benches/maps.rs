use ordered_vecmap::OrderedVecMap;

use std::collections::{BTreeMap, HashMap};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;

pub fn get_trivial(c: &mut Criterion) {
    let n = 1024 * 1024 * 16;

    let mut rng = rand::thread_rng();
    let data = {
        let mut v: Vec<u32> = vec![0; n];
        rng.fill(&mut *v);
        v
    };

    let input = data.first().copied().unwrap();

    {
        let map = data
            .iter()
            .map(|&x| (x, x.to_string()))
            .collect::<OrderedVecMap<_, _>>();

        c.bench_function("ordered-vecmap_get_trivial", |b| {
            b.iter(|| black_box(map.get(black_box(&input)).unwrap()));
        });
    }

    {
        let map = data
            .iter()
            .map(|&x| (x, x.to_string()))
            .collect::<BTreeMap<_, _>>();

        c.bench_function("btreemap_get_trivial", |b| {
            b.iter(|| black_box(map.get(black_box(&input)).unwrap()));
        });
    }

    {
        let map = data
            .iter()
            .map(|&x| (x, x.to_string()))
            .collect::<HashMap<_, _>>();

        c.bench_function("hashmap_get_trivial", |b| {
            b.iter(|| black_box(map.get(black_box(&input)).unwrap()));
        });
    }
}

pub fn get_nontrivial(c: &mut Criterion) {
    let n = 1024 * 128;

    let mut rng = rand::thread_rng();
    let data = {
        let mut v: Vec<u32> = vec![0; n];
        rng.fill(&mut *v);
        v
    };

    let input = data.first().copied().unwrap().to_string();

    {
        let map = data
            .iter()
            .map(|&x| (x.to_string(), x.to_string()))
            .collect::<OrderedVecMap<_, _>>();

        c.bench_function("ordered-vecmap_get_nontrivial", |b| {
            b.iter(|| black_box(map.get(black_box(&input)).unwrap()));
        });
    }

    {
        let map = data
            .iter()
            .map(|&x| (x.to_string(), x.to_string()))
            .collect::<BTreeMap<_, _>>();

        c.bench_function("btreemap_get_nontrivial", |b| {
            b.iter(|| black_box(map.get(black_box(&input)).unwrap()));
        });
    }

    {
        let map = data
            .iter()
            .map(|&x| (x.to_string(), x.to_string()))
            .collect::<HashMap<_, _>>();

        c.bench_function("hashmap_get_nontrivial", |b| {
            b.iter(|| black_box(map.get(black_box(&input)).unwrap()));
        });
    }
}

pub fn build_ordered(c: &mut Criterion) {
    let n = 1024 * 512;

    let mut rng = rand::thread_rng();
    let data = {
        let mut v: Vec<u32> = vec![0; n];
        rng.fill(&mut *v);
        v.sort_unstable();
        v
    };

    c.bench_function("ordered-vecmap_build_ordered", |b| {
        b.iter(|| {
            let mut map = OrderedVecMap::new();
            for &k in &data {
                map.insert(k, k);
            }
            map
        });
    });

    c.bench_function("btreemap_build_ordered", |b| {
        b.iter(|| {
            let mut map = BTreeMap::new();
            for &k in &data {
                map.insert(k, k);
            }
            map
        });
    });

    c.bench_function("hashmap_build_ordered", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for &k in &data {
                map.insert(k, k);
            }
            map
        });
    });
}

pub fn build_unordered(c: &mut Criterion) {
    let n = 1024 * 16;

    let mut rng = rand::thread_rng();
    let data = {
        let mut v: Vec<u32> = vec![0; n];
        rng.fill(&mut *v);
        v
    };

    c.bench_function("ordered-vecmap_build_unordered", |b| {
        b.iter(|| {
            let mut map = OrderedVecMap::new();
            for &k in &data {
                map.insert(k, k);
            }
            map
        });
    });

    c.bench_function("btreemap_build_unordered", |b| {
        b.iter(|| {
            let mut map = BTreeMap::new();
            for &k in &data {
                map.insert(k, k);
            }
            map
        });
    });

    c.bench_function("hashmap_build_unordered", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for &k in &data {
                map.insert(k, k);
            }
            map
        });
    });
}

criterion_group!(
    benches,
    get_trivial,
    get_nontrivial,
    build_ordered,
    build_unordered
);
criterion_main!(benches);
