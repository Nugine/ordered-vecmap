use fnv::FnvHashMap;
use ordered_vecmap::OrderedVecMap;

use std::collections::{BTreeMap, HashMap};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::Rng;

pub fn get_trivial(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_trivial");
    for n in [3, 5, 16, 1024, 4 * 1024, 16 * 1024] {
        let mut rng = rand::thread_rng();
        let data = {
            let mut v: Vec<u64> = vec![0; n];
            rng.fill(&mut *v);
            v
        };

        let input = data.first().copied().unwrap();

        {
            let map = data
                .iter()
                .map(|&x| (x, x.to_string()))
                .collect::<OrderedVecMap<_, _>>();

            let id = BenchmarkId::new("ordered-vecmap", n);
            group.bench_with_input(id, &input, |b, &i| {
                b.iter(|| black_box(map.get(black_box(&i)).unwrap()));
            });
        }

        {
            let map = data
                .iter()
                .map(|&x| (x, x.to_string()))
                .collect::<BTreeMap<_, _>>();

            let id = BenchmarkId::new("btreemap", n);
            group.bench_with_input(id, &input, |b, i| {
                b.iter(|| black_box(map.get(black_box(i)).unwrap()));
            });
        }

        {
            let map = data
                .iter()
                .map(|&x| (x, x.to_string()))
                .collect::<HashMap<_, _>>();

            let id = BenchmarkId::new("hashmap", n);
            group.bench_with_input(id, &input, |b, i| {
                b.iter(|| black_box(map.get(black_box(i)).unwrap()));
            });
        }

        {
            let map = data
                .iter()
                .map(|&x| (x, x.to_string()))
                .collect::<FnvHashMap<_, _>>();

            let id = BenchmarkId::new("fnvhashmap", n);
            group.bench_with_input(id, &input, |b, i| {
                b.iter(|| black_box(map.get(black_box(i)).unwrap()));
            });
        }
    }
}

pub fn get_nontrivial(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_nontrivial");
    for n in [16, 1024, 4 * 1024, 16 * 1024] {
        let mut rng = rand::thread_rng();
        let data = {
            let mut v: Vec<u64> = vec![0; n];
            rng.fill(&mut *v);
            v
        };

        let input = data.first().copied().unwrap().to_string();

        {
            let map = data
                .iter()
                .map(|&x| (x.to_string(), x.to_string()))
                .collect::<OrderedVecMap<_, _>>();

            let id = BenchmarkId::new("ordered-vecmap", n);
            group.bench_with_input(id, &input, |b, i| {
                b.iter(|| black_box(map.get(black_box(i)).unwrap()));
            });
        }

        {
            let map = data
                .iter()
                .map(|&x| (x.to_string(), x.to_string()))
                .collect::<BTreeMap<_, _>>();
            let id = BenchmarkId::new("btreemap", n);
            group.bench_with_input(id, &input, |b, i| {
                b.iter(|| black_box(map.get(black_box(i)).unwrap()));
            });
        }

        {
            let map = data
                .iter()
                .map(|&x| (x.to_string(), x.to_string()))
                .collect::<HashMap<_, _>>();
            let id = BenchmarkId::new("hashmap", n);
            group.bench_with_input(id, &input, |b, i| {
                b.iter(|| black_box(map.get(black_box(i)).unwrap()));
            });
        }
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
