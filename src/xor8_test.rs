use rand::{prelude::random, rngs::SmallRng, Rng, SeedableRng};

use super::*;

fn generate_unique_keys(rng: &mut SmallRng, size: usize) -> Vec<u64> {
    let mut keys: Vec<u64> = Vec::with_capacity(size);
    keys.resize(size, u64::default());

    for key in keys.iter_mut() {
        *key = rng.gen();
    }
    keys.sort_unstable();
    keys.dedup();

    for _i in 0..(size - keys.len()) {
        let key = rng.gen::<u64>();
        if !keys.contains(&key) {
            keys.push(key)
        }
    }

    keys
}

fn test_xor8_build<H>(name: &str, seed: u128, size: u32)
where
    H: Default + BuildHasher,
{
    let (x, y) = {
        let size = size as usize;
        (size / 3, size / 3)
    };

    println!("test_xor8_build<{}> size:{}", name, size);
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let mut filter = Xor8::<H>::new();
    let keys = generate_unique_keys(&mut rng, size as usize);
    let (keys1, keys2, keys3) = (&keys[0..x], &keys[x..x + y], &keys[x + y..]);

    // populate api
    filter.populate(keys1);
    // populate_keys api
    let digests: Vec<u64> = keys2
        .iter()
        .map(|k| {
            let mut hasher = filter.get_hasher();
            k.hash(&mut hasher);
            hasher.finish()
        })
        .collect();
    filter.populate_keys(&digests);
    // insert api
    keys3.iter().for_each(|key| filter.insert(key));

    filter.build().expect("failed build");

    // contains api
    for key in keys.iter() {
        assert!(filter.contains(key), "key {} not present", key);
    }
    // contains_key api
    for key in keys.iter() {
        let digest = {
            let mut hasher = filter.get_hasher();
            key.hash(&mut hasher);
            hasher.finish()
        };
        assert!(filter.contains_key(digest), "key {} not present", key);
    }

    // print some statistics
    let (falsesize, mut matches) = (10_000_000, 0_f64);
    let bpv = (filter.finger_prints.len() as f64) * 8.0 / (keys.len() as f64);
    println!("test_xor8_build<{}> bits per entry {} bits", name, bpv);
    if size > 1000 {
        assert!(bpv < 12.0, "bpv({}) >= 12.0", bpv);
    }

    for _ in 0..falsesize {
        if filter.contains(&rng.gen::<u64>()) {
            matches += 1_f64;
        }
    }

    let fpp = matches * 100.0 / (falsesize as f64);
    println!("test_xor8_build<{}> false positive rate {}%", name, fpp);
    assert!(fpp < 0.40, "fpp({}) >= 0.40", fpp);
}

fn test_xor8_build_keys<H>(name: &str, seed: u128, size: u32)
where
    H: Default + BuildHasher,
{
    println!("test_xor8_build_keys<{}> size:{}", name, size);
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let mut filter = Xor8::<H>::new();

    // build_keys api
    let keys = generate_unique_keys(&mut rng, size as usize);
    let digests: Vec<u64> = keys
        .iter()
        .map(|k| {
            let mut hasher = filter.get_hasher();
            k.hash(&mut hasher);
            hasher.finish()
        })
        .collect();
    filter.build_keys(&digests).expect("failed build_keys");

    // contains api
    for key in keys.iter() {
        assert!(filter.contains(key), "key {} not present", key);
    }
    // contains_key api
    for digest in digests.into_iter() {
        assert!(filter.contains_key(digest), "digest {} not present", digest);
    }

    // print some statistics
    let (falsesize, mut matches) = (10_000_000, 0_f64);
    let bpv = (filter.finger_prints.len() as f64) * 8.0 / (keys.len() as f64);
    println!("test_xor8_build_keys<{}> bits per entry {} bits", name, bpv);
    if size > 1000 {
        assert!(bpv < 12.0, "bpv({}) >= 12.0", bpv);
    }

    for _ in 0..falsesize {
        if filter.contains(&rng.gen::<u64>()) {
            matches += 1_f64;
        }
    }

    let fpp = matches * 100.0 / (falsesize as f64);
    println!(
        "test_xor8_build_keys<{}> false positive rate {}%",
        name, fpp
    );
    assert!(fpp < 0.40, "fpp({}) >= 0.40", fpp);
}

#[test]
fn test_xor8() {
    let mut seed: u128 = random();
    println!("test_xor8 seed:{}", seed);

    for size in [0, 1, 2, 10, 1000, 10_000, 100_000, 1_000_000, 10_000_000].iter() {
        seed = seed.wrapping_add(*size as u128);
        test_xor8_build::<RandomState>("RandomState", seed, *size);
        test_xor8_build::<BuildHasherDefault>("BuildHasherDefault", seed, *size);
        test_xor8_build_keys::<RandomState>("RandomState", seed, *size);
        test_xor8_build_keys::<BuildHasherDefault>("BuildHasherDefault", seed, *size);
    }
}

#[test]
#[ignore]
fn test_xor8_billion() {
    let seed: u128 = random();
    println!("test_xor8_billion seed:{}", seed);

    let size = 1_000_000_000;
    test_xor8_build::<RandomState>("RandomState", seed, size);
    test_xor8_build::<BuildHasherDefault>("BuildHasherDefault", seed, size);
    test_xor8_build_keys::<RandomState>("RandomState", seed, size);
    test_xor8_build_keys::<BuildHasherDefault>("BuildHasherDefault", seed, size);
}

#[cfg(feature = "cbordata")]
#[test]
fn test_xor8_cbor() {
    let seed: u128 = random();
    println!("test_xor8_cbor seed:{}", seed);
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let keys: Vec<u64> = (0..100_000).map(|_| rng.gen::<u64>()).collect();

    let filter = {
        let mut filter = Xor8::<BuildHasherDefault>::new();
        filter.populate(&keys);
        filter.build().expect("fail building xor8 filter");
        filter
    };

    for key in keys.iter() {
        assert!(filter.contains(key), "key {} not present", key);
    }

    let filter = {
        let val = filter.into_cbor().unwrap();
        Xor8::<BuildHasherDefault>::from_cbor(val).unwrap()
    };

    for key in keys.iter() {
        assert!(filter.contains(key), "key {} not present", key);
    }
}
