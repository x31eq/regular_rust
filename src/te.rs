extern crate nalgebra as na;
use na::{DMatrix, DVector};

use super::cangwu;
use super::{Cents, ETMap, FactorElement, Mapping, Tuning};
use cangwu::{rms_of_matrix, TenneyWeighted};

pub struct TETemperament {
    plimit: DVector<Cents>,
    pub melody: Mapping,
    pub tuning: Tuning,
}

impl cangwu::TemperamentClass for TETemperament {
    fn mapping(&'_ self) -> &'_ Mapping {
        &self.melody
    }
}

impl cangwu::TenneyWeighted for TETemperament {
    fn mapping(&'_ self) -> &'_ Mapping {
        &self.melody
    }

    fn plimit(&'_ self) -> &'_ DVector<Cents> {
        &self.plimit
    }
}

impl TETemperament {
    /// Upgrade vectors into a struct of nalgebra objects
    pub fn new(plimit: &[Cents], melody: &[ETMap]) -> Self {
        let plimit = DVector::from_vec(plimit.to_vec());
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
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        self.error() * max_harmonic / 12e2
    }

    pub fn badness(&self) -> Cents {
        let rank = self.melody.len();
        let dimension = self.plimit.len();
        let m = self.weighted_mapping();
        let offset_vec: Vec<f64> = m.row_mean().iter().cloned().collect();
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
        self.tuning.iter().map(|x| x / self.stretch()).collect()
    }

    /// Strictly, pure equivalence interval TE
    pub fn pote_tuning_map(&self) -> Tuning {
        self.tuning_map()
            .iter()
            .map(|x| x / self.stretch())
            .collect()
    }

    pub fn generators_from_primes(&self, interval: &ETMap) -> ETMap {
        self.melody
            .iter()
            .map(|mapping| {
                mapping
                    .iter()
                    .zip(interval.iter())
                    .map(|(&x, &y)| x * y)
                    .sum()
            })
            .collect()
    }

    pub fn pitch_from_steps(&self, interval: &ETMap) -> Cents {
        self.tuning
            .iter()
            .zip(interval)
            .map(|(&x, &y)| x * y as Cents)
            .sum()
    }

    pub fn pitch_from_primes(&self, interval: &ETMap) -> Cents {
        self.pitch_from_steps(&self.generators_from_primes(interval))
    }

    /// Strictly, pure equivalence interval TE
    pub fn pote_mistunings(&self) -> Tuning {
        let tuning_map = self.pote_tuning_map();
        let comparison = tuning_map.iter().zip(self.plimit.iter());
        comparison.map(|(&x, y)| x - y).collect()
    }

    /// Fokker block as steps as integers, not pitches.
    /// This might not actually be a periodicity block
    /// because there's no check on n_pitches
    pub fn fokker_block_steps(&self, n_pitches: FactorElement) -> Mapping {
        let octaves = self.melody.iter().map(|row| row[0]).collect();
        fokker_block(n_pitches, octaves)
    }

    /// This might not actually be a periodicity block
    /// because there's no check on n_pitches
    pub fn fokker_block_pitches(&self, n_pitches: FactorElement) -> Tuning {
        self.fokker_block_steps(n_pitches)
            .iter()
            .cloned()
            .map(|interval| self.pitch_from_steps(&interval))
            .collect()
    }
}

/// A maximally even d from n scale
fn maximally_even(
    d: FactorElement,
    n: FactorElement,
    rotation: FactorElement,
) -> ETMap {
    // Nothing can be negative because of the way / and % work
    assert!(d >= 0);
    assert!(n > 0);
    assert!(rotation >= 0);
    let mut raw_scale = (rotation..=d + rotation).map(|i| (i * n) / d);
    if let Some(tonic) = raw_scale.next() {
        raw_scale.map(|pitch| pitch - tonic).collect()
    }
    else {
        Vec::new()
    }
}

fn fokker_block(n_pitches: FactorElement, octaves: ETMap) -> Mapping {
    assert!(!octaves.is_empty());
    let scales: Mapping = octaves
        .iter()
        .map(|&m| maximally_even(n_pitches, m, 1))
        .collect();
    (0..n_pitches)
        .map(|pitch| {
            assert!(pitch >= 0);
            scales.iter().map(|scale| scale[pitch as usize]).collect()
        })
        .collect()
}

#[cfg(test)]
fn make_marvel() -> TETemperament {
    let marvel_vector = vec![
        vec![22, 35, 51, 62, 76],
        vec![31, 49, 72, 87, 107],
        vec![41, 65, 95, 115, 142],
    ];
    let limit11 = super::PrimeLimit::new(11);
    TETemperament::new(&limit11.pitches, &marvel_vector)
}

#[cfg(test)]
fn make_jove() -> TETemperament {
    let jove_vector = vec![
        vec![27, 43, 63, 76, 94],
        vec![31, 49, 72, 87, 107],
        vec![41, 65, 95, 115, 142],
    ];
    let limit11 = super::PrimeLimit::new(11);
    TETemperament::new(&limit11.pitches, &jove_vector)
}

#[test]
fn complexity() {
    let marvel = make_marvel();
    assert!(0.155663 < marvel.complexity());
    assert!(marvel.complexity() < 0.155664);

    let jove = make_jove();
    // Less precision here because it disagrees with Python.
    assert!(0.17475 < jove.complexity());
    assert!(jove.complexity() < 0.174755);
}

#[test]
fn error() {
    let marvel = make_marvel();
    assert!(0.43069 < marvel.error());
    assert!(marvel.error() < 0.43070);

    let jove = make_jove();
    assert!(0.30486 < jove.error());
    assert!(jove.error() < 0.30487);
}

#[test]
fn tuning() {
    let marvel = make_marvel();
    let expected = "3.96487 17.32226 14.05909";
    let fmt_tuning = format_float_vec(&marvel.tuning, 5);
    assert_eq!(fmt_tuning, expected.to_string());

    let jove = make_jove();
    let expected = "6.00023 17.78766 11.87013";
    let fmt_tuning = format_float_vec(&jove.tuning, 5);
    assert_eq!(fmt_tuning, expected.to_string());
}

#[test]
fn tuning_map() {
    let marvel = make_marvel();
    let expected = "1200.640 1901.403 2785.025 3369.655 4151.204";
    let fmt_tuning = format_float_vec(&marvel.tuning_map(), 3);
    assert_eq!(fmt_tuning, expected.to_string());

    let jove = make_jove();
    let expected = "1200.099 1901.163 2786.388 3368.609 4152.859";
    let fmt_tuning = format_float_vec(&jove.tuning_map(), 3);
    assert_eq!(fmt_tuning, expected.to_string());
}

#[test]
fn mistunings() {
    let marvel = make_marvel();
    let expected = "0.640 -0.552 -1.288 0.829 -0.114";
    let fmt_tuning = format_float_vec(&marvel.mistunings(), 3);
    assert_eq!(fmt_tuning, expected.to_string());

    let jove = make_jove();
    let expected = "0.099 -0.792 0.074 -0.217 1.541";
    let fmt_tuning = format_float_vec(&jove.mistunings(), 3);
    assert_eq!(fmt_tuning, expected.to_string());
}

#[test]
fn pote_tuning() {
    let marvel = make_marvel();
    let expected = "3.96276 17.31303 14.05160";
    let fmt_tuning = format_float_vec(&marvel.pote_tuning(), 5);
    assert_eq!(fmt_tuning, expected.to_string());

    let jove = make_jove();
    let expected = "5.99973 17.78620 11.86915";
    let fmt_tuning = format_float_vec(&jove.pote_tuning(), 5);
    assert_eq!(fmt_tuning, expected.to_string());
}

#[test]
fn pote_tuning_map() {
    let marvel = make_marvel();
    let expected = "1200.000 1900.389 2783.540 3367.858 4148.990";
    let fmt_tuning = format_float_vec(&marvel.pote_tuning_map(), 3);
    assert_eq!(fmt_tuning, expected.to_string());

    let jove = make_jove();
    let expected = "1200.000 1901.007 2786.159 3368.331 4152.517";
    let fmt_tuning = format_float_vec(&jove.pote_tuning_map(), 3);
    assert_eq!(fmt_tuning, expected.to_string());
}

#[test]
fn pote_mistunings() {
    let marvel = make_marvel();
    let expected = "0.000 -1.566 -2.773 -0.968 -2.328";
    let fmt_tuning = format_float_vec(&marvel.pote_mistunings(), 3);
    assert_eq!(fmt_tuning, expected.to_string());

    let jove = make_jove();
    let expected = "0.000 -0.948 -0.155 -0.495 1.199";
    let fmt_tuning = format_float_vec(&jove.pote_mistunings(), 3);
    assert_eq!(fmt_tuning, expected.to_string());
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
    assert!(4.83894 < mystery.complexity());
    assert!(mystery.complexity() < 4.83895);
    assert!(0.51238 < mystery.error());
    assert!(mystery.error() < 0.51239);
    assert!(1.89606 < mystery.adjusted_error());
    assert!(mystery.adjusted_error() < 1.89607);

    let fmt_tuning_map = format_float_vec(&mystery.tuning_map(), 3);
    let expected = "1199.507 1902.667 2787.209 3366.282 4152.166 4441.702";
    assert_eq!(fmt_tuning_map, expected.to_string());

    let fmt_errors = format_float_vec(&mystery.mistunings(), 3);
    let expected = "-0.493 0.712 0.896 -2.544 0.848 1.175";
    assert_eq!(fmt_errors, expected.to_string());

    let fmt_tuning_map = format_float_vec(&mystery.pote_tuning_map(), 3);
    let expected = "1200.000 1903.448 2788.354 3367.664 4153.871 4443.527";
    assert_eq!(fmt_tuning_map, expected.to_string());
}

#[test]
fn test_maximally_even() {
    assert_eq!(maximally_even(7, 12, 0), vec![1, 3, 5, 6, 8, 10, 12]);
    assert_eq!(maximally_even(7, 12, 5), vec![2, 4, 5, 7, 9, 11, 12]);
    assert_eq!(maximally_even(5, 19, 3), vec![4, 8, 12, 15, 19]);
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
            vec![3, 6],
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
fn rt_fokker_block() {
    let marvel = make_marvel();
    assert_eq!(
        marvel.fokker_block_steps(22),
        vec![
            vec![1, 2, 2],
            vec![2, 3, 4],
            vec![3, 5, 6],
            vec![4, 6, 8],
            vec![5, 8, 10],
            vec![6, 9, 12],
            vec![7, 10, 14],
            vec![8, 12, 15],
            vec![9, 13, 17],
            vec![10, 15, 19],
            vec![11, 16, 21],
            vec![12, 17, 23],
            vec![13, 19, 25],
            vec![14, 20, 27],
            vec![15, 22, 28],
            vec![16, 23, 30],
            vec![17, 24, 32],
            vec![18, 26, 34],
            vec![19, 27, 36],
            vec![20, 29, 38],
            vec![21, 30, 40],
            vec![22, 31, 41],
        ]
    );
    assert_eq!(
        marvel.fokker_block_steps(7),
        vec![
            vec![4, 5, 6],
            vec![7, 9, 12],
            vec![10, 14, 18],
            vec![13, 18, 24],
            vec![16, 23, 30],
            vec![19, 27, 36],
            vec![22, 31, 41],
        ]
    );
}

#[test]
fn tuned_block() {
    let block = make_marvel().fokker_block_pitches(22);
    let fmt_block = format_float_vec(&block, 3);
    assert_eq!(fmt_block, "66.728 116.133 182.861 232.266 298.993 348.399 397.804 450.473 499.878 566.605 616.011 665.416 732.144 781.549 834.218 883.623 933.028 999.756 1049.161 1115.889 1165.294 1200.640")
}

#[test]
fn generators() {
    let marvel = make_marvel();
    let twotoe = marvel.generators_from_primes(&vec![3, 0, 0, -1, 0]);
    assert_eq!(twotoe, vec![4, 6, 8]);
}

#[test]
fn pitches() {
    let marvel = make_marvel();
    let twotoe = marvel.pitch_from_steps(&vec![4, 6, 8]);
    assert_eq!(format!("{:.3}", twotoe), "232.266");

    let twotoe = marvel.pitch_from_primes(&vec![3, 0, 0, -1, 0]);
    assert_eq!(format!("{:.3}", twotoe), "232.266");
    let twotoe = marvel.pitch_from_primes(&vec![-3, 0, 0, 0, 1]);
    assert_eq!(format!("{:.3}", twotoe), "549.283");
}

#[cfg(test)]
fn format_float_vec(tuning: &Tuning, decimals: usize) -> String {
    let mut result = "".to_string();
    for pitch in tuning.iter() {
        result.push_str(" ");
        result.push_str(&format!("{:.*}", decimals, pitch));
    }
    result.remove(0);
    result
}
