use super::names::NAMES_BY_LIMIT;
use super::{ETMap, Mapping, PrimeLimit};

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
}

// The rest of this file is for testing

#[cfg(test)]
struct StubTemperamentClass {
    pub melody: Mapping,
}

#[cfg(test)]
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
fn rank() {
    assert_eq!(make_marvel().rank(), 3);
    assert_eq!(make_jove().rank(), 3);
}
