extern crate nalgebra as na;
use na::{DMatrix, DVector};

use super::cangwu::{CangwuTemperament, TenneyWeighted, rms_of_matrix};
use super::temperament_class::TemperamentClass;
use super::{
    Cents, ETMap, ETSlice, Exponent, Mapping, PrimeLimit, Tuning, map,
    mapping_from_name,
};

pub struct TETemperament<'a> {
    plimit: &'a [Cents],
    pub melody: Mapping,
    pub tuning: Tuning,
}

impl TemperamentClass for TETemperament<'_> {
    fn mapping(&self) -> &Mapping {
        &self.melody
    }
}

impl TenneyWeighted for TETemperament<'_> {
    fn mapping(&self) -> &Mapping {
        &self.melody
    }

    fn plimit(&self) -> &[Cents] {
        self.plimit
    }
}

impl<'a> TETemperament<'a> {
    /// Upgrade vectors into a struct of nalgebra objects
    pub fn new(plimit: &'a [Cents], melody: &[ETMap]) -> Self {
        let melody = melody.to_vec();
        let mut rt = TETemperament {
            plimit,
            melody,
            tuning: vec![0.0],
        };
        let wmap = rt.weighted_mapping();
        let pinv = wmap.pseudo_inverse(0.0).expect("no pseudoinverse");
        let tuning = pinv.column_sum() * 12e2;
        rt.tuning = tuning.iter().cloned().collect();
        rt
    }

    /// Turn an ET name like "12 & 19" into a temperament object
    pub fn from_name(plimit: &'a PrimeLimit, name: &str) -> Option<Self> {
        mapping_from_name(plimit, name)
            .map(|mapping| TETemperament::new(&plimit.pitches, &mapping))
    }

    pub fn error(&self) -> f64 {
        self.badness() / self.complexity()
    }

    pub fn complexity(&self) -> f64 {
        rms_of_matrix(&self.weighted_mapping())
    }

    pub fn adjusted_error(&self) -> f64 {
        let max_harmonic = self
            .plimit
            .iter()
            .max_by(|a, b| a.partial_cmp(b).expect("Incomparable harmonic"))
            .expect("No max harmonic");
        self.error() * max_harmonic / 12e2
    }

    pub fn badness(&self) -> Cents {
        let rank = self.melody.len();
        let dimension = self.plimit.len();
        let m = self.weighted_mapping();
        let offset_vec: Vec<_> = m.row_mean().iter().cloned().collect();
        let mut translation = DMatrix::from_vec(rank, 1, offset_vec.clone());
        assert!(dimension > 0);
        for _ in 1..dimension {
            translation.extend(offset_vec.clone());
        }
        rms_of_matrix(&(m - translation.transpose())) * 1200.0
    }

    pub fn tuning_map(&self) -> Tuning {
        let rank = self.melody.len();
        let dimension = self.plimit.len();
        let tuning = DVector::from_vec(self.tuning.clone());
        let mapping = &self.melody;
        let flattened = mapping
            .iter()
            .flat_map(|mapping| mapping.iter().map(|&m| m as f64));
        let melody = DMatrix::from_iterator(dimension, rank, flattened);
        (melody * tuning).iter().cloned().collect()
    }

    pub fn mistunings(&self) -> Tuning {
        let tuning_map = self.tuning_map();
        let comparison = tuning_map.iter().zip(self.plimit.iter());
        comparison.map(|(&x, y)| x - y).collect()
    }

    fn stretch(&self) -> f64 {
        self.tuning_map()[0] / self.plimit[0]
    }

    /// Strictly, pure equivalence interval TE
    pub fn pote_tuning(&self) -> Tuning {
        map(|x| x / self.stretch(), &self.tuning)
    }

    /// Strictly, pure equivalence interval TE
    pub fn pote_tuning_map(&self) -> Tuning {
        map(|x| x / self.stretch(), &self.tuning_map())
    }

    pub fn generators_from_primes(&self, interval: &ETSlice) -> ETMap {
        map(
            |mapping| {
                mapping
                    .iter()
                    .zip(interval.iter())
                    .map(|(&x, &y)| x * y)
                    .sum()
            },
            &self.melody,
        )
    }

    pub fn pitch_from_steps(&self, interval: &ETSlice) -> Cents {
        self.tuning
            .iter()
            .zip(interval)
            .map(|(&x, &y)| x * y as Cents)
            .sum()
    }

    pub fn pitch_from_primes(&self, interval: &ETSlice) -> Cents {
        self.pitch_from_steps(&self.generators_from_primes(interval))
    }

    /// Strictly, pure equivalence interval TE
    pub fn pote_mistunings(&self) -> Tuning {
        let tuning_map = self.pote_tuning_map();
        let comparison = tuning_map.iter().zip(self.plimit.iter());
        comparison.map(|(&x, y)| x - y).collect()
    }

    pub fn unison_vectors(&self, n_results: usize) -> Mapping {
        let tc = CangwuTemperament::new(self.plimit, &self.melody);
        tc.unison_vectors(self.error(), n_results)
    }

    /// Fokker block as steps as integers, not pitches.
    /// This might not actually be a periodicity block
    /// because there's no check on n_pitches
    pub fn fokker_block_steps(&self, n_pitches: Exponent) -> Mapping {
        let octaves = map(|row| row[0], &self.melody);
        fokker_block(n_pitches, octaves)
    }

    /// This might not actually be a periodicity block
    /// because there's no check on n_pitches
    pub fn fokker_block_pitches(&self, n_pitches: Exponent) -> Tuning {
        self.fokker_block_steps(n_pitches)
            .iter()
            .map(|interval| self.pitch_from_steps(interval))
            .collect()
    }
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

#[cfg(test)]
fn make_marvel(limit11: &super::PrimeLimit) -> TETemperament<'_> {
    let marvel_vector = vec![
        vec![22, 35, 51, 62, 76],
        vec![31, 49, 72, 87, 107],
        vec![41, 65, 95, 115, 142],
    ];
    TETemperament::new(&limit11.pitches, &marvel_vector)
}

#[cfg(test)]
fn make_jove(limit11: &super::PrimeLimit) -> TETemperament<'_> {
    let jove_vector = vec![
        vec![27, 43, 63, 76, 94],
        vec![31, 49, 72, 87, 107],
        vec![41, 65, 95, 115, 142],
    ];
    TETemperament::new(&limit11.pitches, &jove_vector)
}

#[test]
fn complexity() {
    let limit11 = super::PrimeLimit::new(11);
    let marvel = make_marvel(&limit11);
    super::assert_between!(0.155663, marvel.complexity(), 0.155664);

    let jove = make_jove(&limit11);
    // Less precision here because it disagrees with Python.
    super::assert_between!(0.17475, jove.complexity(), 0.174755);
}

#[test]
fn error() {
    let limit11 = super::PrimeLimit::new(11);
    let marvel = make_marvel(&limit11);
    super::assert_between!(0.43069, marvel.error(), 0.43070);

    let jove = make_jove(&limit11);
    super::assert_between!(0.30486, jove.error(), 0.30487);
}

#[test]
fn tuning() {
    let limit11 = super::PrimeLimit::new(11);
    let marvel = make_marvel(&limit11);
    let expected = "3.96487 17.32226 14.05909";
    check_float_vec(&marvel.tuning, 5, expected);

    let jove = make_jove(&limit11);
    let expected = "6.00023 17.78766 11.87013";
    check_float_vec(&jove.tuning, 5, expected);
}

#[test]
fn tuning_map() {
    let limit11 = super::PrimeLimit::new(11);
    let marvel = make_marvel(&limit11);
    let expected = "1200.640 1901.403 2785.025 3369.655 4151.204";
    check_float_vec(&marvel.tuning_map(), 3, expected);

    let jove = make_jove(&limit11);
    let expected = "1200.099 1901.163 2786.388 3368.609 4152.859";
    check_float_vec(&jove.tuning_map(), 3, expected);
}

#[test]
fn mistunings() {
    let limit11 = super::PrimeLimit::new(11);
    let marvel = make_marvel(&limit11);
    let expected = "0.640 -0.552 -1.288 0.829 -0.114";
    check_float_vec(&marvel.mistunings(), 3, expected);

    let jove = make_jove(&limit11);
    let expected = "0.099 -0.792 0.074 -0.217 1.541";
    check_float_vec(&jove.mistunings(), 3, expected);
}

#[test]
fn pote_tuning() {
    let limit11 = super::PrimeLimit::new(11);
    let marvel = make_marvel(&limit11);
    let expected = "3.96276 17.31303 14.05160";
    check_float_vec(&marvel.pote_tuning(), 5, expected);

    let jove = make_jove(&limit11);
    let expected = "5.99973 17.78620 11.86915";
    check_float_vec(&jove.pote_tuning(), 5, expected);
}

#[test]
fn pote_tuning_map() {
    let limit11 = super::PrimeLimit::new(11);
    let marvel = make_marvel(&limit11);
    let expected = "1200.000 1900.389 2783.540 3367.858 4148.990";
    check_float_vec(&marvel.pote_tuning_map(), 3, expected);

    let jove = make_jove(&limit11);
    let expected = "1200.000 1901.007 2786.159 3368.331 4152.517";
    check_float_vec(&jove.pote_tuning_map(), 3, expected);
}

#[test]
fn pote_mistunings() {
    let limit11 = super::PrimeLimit::new(11);
    let marvel = make_marvel(&limit11);
    let expected = "0.000 -1.566 -2.773 -0.968 -2.328";
    check_float_vec(&marvel.pote_mistunings(), 3, expected);

    let jove = make_jove(&limit11);
    let expected = "0.000 -0.948 -0.155 -0.495 1.199";
    check_float_vec(&jove.pote_mistunings(), 3, expected);
}

#[test]
fn marvel_from_name() {
    let limit11 = super::PrimeLimit::new(11);
    let expected = make_marvel(&limit11);
    let from_name = TETemperament::from_name(&limit11, "22 & 31 & 41")
        .expect("couldn't make marvel from name");
    assert_eq!(expected.melody, from_name.melody);
    assert_eq!(expected.plimit, from_name.plimit);
}

#[rustfmt::skip]
#[test]
fn mystery() {
    let mystery_vector = vec![
        vec![29, 46, 67, 81, 100, 107],
        vec![58, 92, 135, 163, 201, 215],
    ];
    let limit13 = super::PrimeLimit::new(13);
    let mystery = TETemperament::new(&limit13.pitches, &mystery_vector);
    super::assert_between!(4.83894, mystery.complexity(), 4.83895);
    super::assert_between!(0.51238, mystery.error(), 0.51239);
    super::assert_between!(1.89606, mystery.adjusted_error(), 1.89607);

    let expected = "1199.507 1902.667 2787.209 3366.282 4152.166 4441.702";
    check_float_vec(&mystery.tuning_map(), 3, expected);

    let expected = "-0.493 0.712 0.896 -2.544 0.848 1.175";
    check_float_vec(&mystery.mistunings(), 3, expected);

    let expected = "1200.000 1903.448 2788.354 3367.664 4153.871 4443.527";
    check_float_vec(&mystery.pote_tuning_map(), 3, expected);
}

#[test]
fn marvel_unison_vectors() {
    let limit = super::PrimeLimit::new(11);
    let lt = make_marvel(&limit);
    let n_results = 3;
    let uvs = lt.unison_vectors(n_results);
    assert_eq!(uvs.len(), n_results);
    assert!(uvs.contains(&vec![2, 3, 1, -2, -1]));
    assert!(uvs.contains(&vec![-5, 2, 2, -1, 0]));
    assert!(uvs.contains(&vec![-7, -1, 1, 1, 1]));
}

#[test]
fn porcupine_unison_vectors() {
    let limit = super::PrimeLimit::new(11);
    let porcupine_vector =
        vec![vec![22, 35, 51, 62, 76], vec![15, 24, 35, 42, 52]];
    let lt = TETemperament::new(&limit.pitches, &porcupine_vector);
    let n_results = 5;
    let uvs = lt.unison_vectors(n_results);
    assert_eq!(uvs.len(), n_results);
    assert!(uvs.contains(&vec![-1, -3, 1, 0, 1]));
    assert!(uvs.contains(&vec![6, -2, 0, -1, 0]));
    assert!(uvs.contains(&vec![2, -2, 2, 0, -1]));
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
fn rt_fokker_block() {
    let limit11 = super::PrimeLimit::new(11);
    let marvel = make_marvel(&limit11);
    assert_eq!(
        marvel.fokker_block_steps(22),
        vec![
            vec![1, 1, 2],
            vec![2, 3, 4],
            vec![3, 4, 6],
            vec![4, 6, 8],
            vec![5, 7, 10],
            vec![6, 8, 12],
            vec![7, 10, 13],
            vec![8, 11, 15],
            vec![9, 13, 17],
            vec![10, 14, 19],
            vec![11, 15, 21],
            vec![12, 17, 23],
            vec![13, 18, 25],
            vec![14, 20, 26],
            vec![15, 21, 28],
            vec![16, 22, 30],
            vec![17, 24, 32],
            vec![18, 25, 34],
            vec![19, 27, 36],
            vec![20, 28, 38],
            vec![21, 30, 40],
            vec![22, 31, 41],
        ]
    );
    assert_eq!(
        marvel.fokker_block_steps(7),
        vec![
            vec![3, 4, 6],
            vec![6, 9, 12],
            vec![9, 13, 18],
            vec![12, 18, 24],
            vec![15, 22, 30],
            vec![19, 27, 36],
            vec![22, 31, 41],
        ]
    );
    let empty_scale: Mapping = Vec::new();
    assert_eq!(marvel.fokker_block_steps(0), empty_scale);
}

#[test]
fn tuned_block() {
    let limit11 = super::PrimeLimit::new(11);
    let block = make_marvel(&limit11).fokker_block_pitches(22);
    let expected = "49.405 116.133 165.538 232.266 281.671 331.076 383.745 \
        433.150 499.878 549.283 598.689 665.416 714.821 767.490 \
        816.895 866.301 933.028 982.434 1049.161 1098.566 1165.294 \
        1200.640";
    check_float_vec(&block, 3, expected);
}

#[test]
fn generators() {
    let limit11 = super::PrimeLimit::new(11);
    let marvel = make_marvel(&limit11);
    let twotoe = marvel.generators_from_primes(&vec![3, 0, 0, -1, 0]);
    assert_eq!(twotoe, vec![4, 6, 8]);
}

#[test]
fn pitches() {
    let limit11 = super::PrimeLimit::new(11);
    let marvel = make_marvel(&limit11);
    let twotoe = marvel.pitch_from_steps(&vec![4, 6, 8]);
    assert_eq!(format!("{:.3}", twotoe), "232.266");

    let twotoe = marvel.pitch_from_primes(&vec![3, 0, 0, -1, 0]);
    assert_eq!(format!("{:.3}", twotoe), "232.266");
    let twotoe = marvel.pitch_from_primes(&vec![-3, 0, 0, 0, 1]);
    assert_eq!(format!("{:.3}", twotoe), "549.283");
}

#[cfg(test)]
fn check_float_vec(tuning: &Tuning, decimals: usize, expected: &str) {
    let mut formatted = "".to_string();
    for pitch in tuning.iter() {
        formatted.push_str(" ");
        formatted.push_str(&format!("{:.*}", decimals, pitch));
    }
    formatted.remove(0);
    assert_eq!(formatted, expected.to_string());
}
