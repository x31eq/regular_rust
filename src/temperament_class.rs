use super::names::NAMES_BY_LIMIT;
use super::{ETMap, ETSlice, Exponent, Mapping, PrimeLimit, map};

pub trait TemperamentClass {
    fn mapping(&self) -> &Mapping;

    /// Unique identifier for the mapping
    /// (hermite normal form flattened and
    /// with always-zero entries removed)
    fn key(&self) -> ETMap {
        self.reduced_mapping()
            .iter()
            .enumerate()
            .rev()
            .flat_map(|(i, col)| col[i..].iter().cloned())
            .collect()
    }

    fn reduced_mapping(&self) -> Mapping {
        super::hermite_normal_form(self.mapping())
    }

    /// Actual rank of the mapping matrix
    fn rank(&self) -> usize {
        let mut result = 0;
        for col in self.reduced_mapping().iter() {
            if col.iter().any(|&x| x != 0) {
                result += 1;
            }
        }
        result
    }

    fn name(&self, limit: &PrimeLimit) -> Option<&'static str> {
        // It would be easier if the headings were already &str...
        let limit_key: Vec<&str> =
            limit.headings.iter().map(|s| s.as_str()).collect();
        match NAMES_BY_LIMIT.get(&limit_key) {
            Some(names_by_linmap) => {
                names_by_linmap.get(&self.key()).copied()
            }
            None => None,
        }
    }

    fn et_belongs(&self, et: &ETSlice) -> bool {
        let mut melody = self.mapping().clone();
        melody.insert(0, et.to_vec());
        let new_rt = StubTemperamentClass { melody };
        self.rank() == new_rt.rank()
    }

    fn generators_from_primes(&self, interval: &ETSlice) -> ETMap {
        map(
            |mapping| {
                mapping
                    .iter()
                    .zip(interval.iter())
                    .map(|(&x, &y)| x * y)
                    .sum()
            },
            self.mapping(),
        )
    }

    /// Fokker block as steps as integers, not pitches.
    /// This might not actually be a periodicity block
    /// because there's no check on n_pitches
    fn fokker_block_steps(&self, n_pitches: Exponent) -> Mapping {
        let octaves = map(|row| row[0], self.mapping());
        fokker_block(n_pitches, octaves)
    }
}

/// Reverse engineer a key to get a mapping suitable for
/// constructing a temperament class object
pub fn key_to_mapping(n_primes: usize, key: &ETSlice) -> Option<Mapping> {
    let mut result = vec![];
    let mut remaining = key.to_vec();
    while !remaining.is_empty() {
        let mut new_vec = vec![];
        for _ in 0..(n_primes - result.len()) {
            new_vec.insert(0, remaining.pop()?);
        }
        for _ in 0..result.len() {
            new_vec.insert(0, 0);
        }
        result.push(new_vec);
    }
    Some(result)
}

fn fokker_block(n_pitches: Exponent, octaves: ETMap) -> Mapping {
    // Make the first coordinate special
    let columns = octaves.iter().cloned().min().expect("Empty ET map");
    let scales = map(
        |&m| {
            if (m + columns) <= n_pitches && columns != m && columns > 0 {
                let correction = (n_pitches - m) / columns;
                let eff_m = m + columns * correction;
                maximally_even(n_pitches, eff_m, 1)
                    .iter()
                    .zip(maximally_even(n_pitches, columns, 1))
                    .map(|(&x, y)| x - correction * y)
                    .collect()
            } else {
                maximally_even(n_pitches, m, 1)
            }
        },
        &octaves,
    );
    (0..n_pitches)
        .map(|pitch| {
            assert!(pitch >= 0);
            map(|scale| scale[pitch as usize], &scales)
        })
        .collect()
}

/// A maximally even d from n scale
fn maximally_even(d: Exponent, n: Exponent, rotation: Exponent) -> ETMap {
    if d == 0 {
        return Vec::new();
    }
    // Nothing can be negative because of the way / and % work
    assert!(d > 0);
    assert!(n >= 0);
    assert!(rotation >= 0);
    let mut raw_scale = (rotation..=d + rotation).map(|i| (i * n) / d);
    let tonic = raw_scale
        .next()
        .expect("Empty maximally even scale: check assertions");
    raw_scale.map(|pitch| pitch - tonic).collect()
}

struct StubTemperamentClass {
    pub melody: Mapping,
}

impl TemperamentClass for StubTemperamentClass {
    fn mapping(&self) -> &Mapping {
        &self.melody
    }
}

#[cfg(test)]
fn make_meantone() -> StubTemperamentClass {
    let meantone_vector = vec![vec![19, 30, 44], vec![31, 49, 72]];
    StubTemperamentClass {
        melody: meantone_vector,
    }
}

#[cfg(test)]
fn make_marvel() -> StubTemperamentClass {
    let marvel_vector = vec![
        vec![22, 35, 51, 62, 76],
        vec![31, 49, 72, 87, 107],
        vec![41, 65, 95, 115, 142],
    ];
    StubTemperamentClass {
        melody: marvel_vector,
    }
}

#[cfg(test)]
fn make_jove() -> StubTemperamentClass {
    let jove_vector = vec![
        vec![27, 43, 63, 76, 94],
        vec![31, 49, 72, 87, 107],
        vec![41, 65, 95, 115, 142],
    ];
    StubTemperamentClass {
        melody: jove_vector,
    }
}

#[rustfmt::skip]
#[test]
fn hermite() {
    let marvel = make_marvel();
    let marvel_hermite =
        vec![[1, 0, 0, -5, 12],
             [0, 1, 0, 2, -1],
             [0, 0, 1, 2, -3]];
    assert_eq!(marvel.reduced_mapping(), marvel_hermite);

    let jove = make_jove();
    let jove_hermite = vec![[1, 1, 1, 2, 2],
                            [0, 2, 1, 1, 5],
                            [0, 0, 2, 1, 0]];
    assert_eq!(jove.reduced_mapping(), jove_hermite);
}

#[rustfmt::skip]
#[test]
fn key() {
    assert_eq!(
        make_marvel().key(),
        vec![1, 2, -3,
          1, 0, 2, -1,
       1, 0, 0, -5, 12]
    );

    assert_eq!(make_jove().key(), vec![2, 1, 0,
                                    2, 1, 1, 5,
                                 1, 1, 1, 2, 2]);
}

#[test]
fn meantone_name() {
    let limit5 = PrimeLimit::new(5);
    let meantone = make_meantone();
    assert_eq!(meantone.name(&limit5), Some("Meantone"));
}

#[test]
fn marvel_name() {
    let limit11 = PrimeLimit::new(11);
    let marvel = make_marvel();
    assert_eq!(marvel.name(&limit11), Some("Marvel"));
}

#[test]
fn jove_name() {
    let limit11 = PrimeLimit::new(11);
    let jove = make_jove();
    assert_eq!(jove.name(&limit11), Some("Jove"));
}

#[test]
fn bad_limit_name() {
    let mut limit5 = PrimeLimit::new(5);
    limit5.headings[0] = "octave".to_string();
    let meantone = make_meantone();
    assert_eq!(meantone.name(&limit5), None);
}

#[test]
fn unlisted_name() {
    let limit5 = PrimeLimit::new(5);
    let melody = vec![vec![1, 2, 3], vec![4, 5, 6]];
    let tc = StubTemperamentClass { melody };
    assert_eq!(tc.name(&limit5), None);
}

#[test]
fn meantone_has_12() {
    let meantone = make_meantone();
    let et = vec![12, 19, 28];
    assert!(meantone.et_belongs(&et));
}

#[test]
fn meantone_has_41() {
    let meantone = make_meantone();
    let et = vec![41, 65, 95];
    assert!(!meantone.et_belongs(&et));
}

#[test]
fn marvel_has_27() {
    let marvel = make_marvel();
    let et = vec![27, 43, 63, 76, 94];
    assert!(!marvel.et_belongs(&et));
}

#[test]
fn marvel_has_72() {
    let marvel = make_marvel();
    let et = vec![72, 114, 167, 202, 249];
    assert!(marvel.et_belongs(&et));
}

#[test]
fn marvel_key_to_mapping() {
    let marvel = make_marvel();
    let key = marvel.key();
    let redmap = marvel.reduced_mapping();
    let dimension = redmap[0].len();
    let poss_mapping = key_to_mapping(dimension, &key);
    assert_eq!(Some(redmap), poss_mapping);
}

#[test]
fn meantone_key_to_mapping() {
    let meantone = make_meantone();
    let key = meantone.key();
    let redmap = meantone.reduced_mapping();
    let dimension = redmap[0].len();
    let poss_mapping = key_to_mapping(dimension, &key);
    assert_eq!(Some(redmap), poss_mapping);
}

#[test]
fn jove_key_to_mapping() {
    let jove = make_jove();
    let key = jove.key();
    let redmap = jove.reduced_mapping();
    let dimension = redmap[0].len();
    let poss_mapping = key_to_mapping(dimension, &key);
    assert_eq!(Some(redmap), poss_mapping);
}

#[test]
fn rank() {
    assert_eq!(make_marvel().rank(), 3);
    assert_eq!(make_jove().rank(), 3);
}

#[test]
fn test_fokker_block() {
    assert_eq!(
        fokker_block(7, vec![7, 12]),
        vec![
            vec![1, 2],
            vec![2, 4],
            vec![3, 5],
            vec![4, 7],
            vec![5, 9],
            vec![6, 11],
            vec![7, 12],
        ]
    );
    assert_eq!(
        fokker_block(6, vec![6, 5, 17]),
        vec![
            vec![1, 1, 3],
            vec![2, 2, 6],
            vec![3, 3, 9],
            vec![4, 4, 12],
            vec![5, 5, 15],
            vec![6, 5, 17],
        ]
    );
    assert_eq!(
        fokker_block(6, vec![5, 17]),
        vec![
            vec![1, 3],
            vec![2, 6],
            vec![3, 9],
            vec![4, 12],
            vec![5, 15],
            vec![5, 17],
        ]
    );
}

#[test]
fn big_fokker_block() {
    // Check that something sensible happens
    // when simple maximally even sets won't do
    assert_eq!(
        fokker_block(12, vec![3, 6]),
        vec![
            vec![0, 1],
            vec![0, 2],
            vec![1, 1],
            vec![1, 2],
            vec![1, 3],
            vec![1, 4],
            vec![2, 3],
            vec![2, 4],
            vec![2, 5],
            vec![2, 6],
            vec![3, 5],
            vec![3, 6],
        ]
    );

    // This should be symmetrical
    assert_eq!(
        fokker_block(12, vec![6, 3]),
        vec![
            vec![1, 0],
            vec![2, 0],
            vec![1, 1],
            vec![2, 1],
            vec![3, 1],
            vec![4, 1],
            vec![3, 2],
            vec![4, 2],
            vec![5, 2],
            vec![6, 2],
            vec![5, 3],
            vec![6, 3],
        ]
    );
}

#[test]
fn test_maximally_even() {
    assert_eq!(maximally_even(7, 12, 0), vec![1, 3, 5, 6, 8, 10, 12]);
    assert_eq!(maximally_even(7, 12, 1), vec![2, 4, 5, 7, 9, 11, 12]);
    assert_eq!(maximally_even(5, 19, 2), vec![4, 8, 12, 15, 19]);
    assert_eq!(maximally_even(3, 0, 0), vec![0, 0, 0]);
    for i in 0..10 {
        assert_eq!(maximally_even(2, 22, i), vec![11, 22]);
    }
    assert_eq!(maximally_even(0, 10, 11).len(), 0);
}
