use microlp::{ComparisonOp, OptimizationDirection, Problem};

use super::cangwu::TenneyWeighted;
use super::temperament_class::TemperamentClass;
use super::tuned_temperament::TunedTemperament;
use super::{Cents, ETMap, Mapping, Tuning};

pub struct TOPTemperament<'a> {
    plimit: &'a [Cents],
    pub melody: Mapping,
    pub tuning: Tuning,
}

impl TemperamentClass for TOPTemperament<'_> {
    fn mapping(&self) -> &Mapping {
        &self.melody
    }
}

impl TunedTemperament for TOPTemperament<'_> {
    fn plimit(&self) -> &[Cents] {
        self.plimit
    }

    fn tuning(&self) -> &Tuning {
        &self.tuning
    }
}

impl TenneyWeighted for TOPTemperament<'_> {
    fn mapping(&self) -> &Mapping {
        &self.melody
    }

    fn plimit(&self) -> &[Cents] {
        self.plimit
    }
}

impl<'a> TOPTemperament<'a> {
    /// Upgrade vectors into a struct of nalgebra objects
    pub fn new(plimit: &'a [Cents], melody: &[ETMap]) -> Self {
        let melody = melody.to_vec();
        let mut rt = TOPTemperament {
            plimit,
            melody: melody.to_vec(),
            tuning: vec![0.0],
        };
        rt.optimize();
        rt
    }

    pub fn error(&self) -> Cents {
        // no .max() for f64
        self.weighted_tuning_map()
            .iter()
            .map(|&m| (m - 1.0).abs())
            .fold(0.0, f64::max)
            * 12e2
    }

    /// TOP-equivalent error for current tuning
    /// after applying an optimal stretch
    pub fn optimal_sretch_error(&self) -> Cents {
        let wmap = self.weighted_tuning_map();
        // no .max() or .min() for f64
        let max = wmap.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let min = wmap.iter().copied().fold(f64::INFINITY, f64::min);
        (max - min) / (max + min) * 12e2
    }

    pub fn optimize(&mut self) {
        let mut problem = Problem::new(OptimizationDirection::Minimize);
        let error = problem.add_var(1.0, (0.0, f64::INFINITY));
        let mut vars = Vec::new();
        for _ in &self.melody {
            vars.push(problem.add_var(0.0, (0.0, f64::INFINITY)));
        }
        for (i, &p) in self.plimit.iter().enumerate() {
            let tuned_prime: Vec<_> = self
                .melody
                .iter()
                .zip(vars.iter())
                .map(|(m, &v)| (v, m[i] as f64))
                .collect();
            // error >= tuned_prime/p - 1 and error >= 1 - tuned_prime/p
            // error*p >= tuned_prime - p and error*p >= p - tuned_prime
            // error*p - tuned_prime >= -p and error*p + tuned_prime >= p
            // tuned_prime - error*p <= p and error*p + tuned_prime >= p
            let mut constraint = tuned_prime.clone();
            constraint.push((error, -p));
            problem.add_constraint(&constraint, ComparisonOp::Le, p);
            constraint.pop();
            constraint.push((error, p));
            problem.add_constraint(&constraint, ComparisonOp::Ge, p);
        }
        let solution = problem.solve().unwrap();
        self.tuning = vars.iter().map(|&v| solution[v]).collect();
    }
}

#[cfg(test)]
fn make_meantone(limit5: &super::PrimeLimit) -> TOPTemperament<'_> {
    let meantone_vector = vec![vec![19, 30, 44], vec![31, 49, 72]];
    TOPTemperament::new(&limit5.pitches, &meantone_vector)
}

#[cfg(test)]
fn make_marvel(limit11: &super::PrimeLimit) -> TOPTemperament<'_> {
    let marvel_vector = vec![
        vec![22, 35, 51, 62, 76],
        vec![31, 49, 72, 87, 107],
        vec![41, 65, 95, 115, 142],
    ];
    TOPTemperament::new(&limit11.pitches, &marvel_vector)
}

#[test]
fn meantone_tuning() {
    let limit5 = super::PrimeLimit::new(5);
    let meantone = make_meantone(&limit5);
    assert_eq!(meantone.tuning.len(), 2);
    super::assert_between!(6.07, meantone.tuning[0], 6.08);
    super::assert_between!(35.03, meantone.tuning[1], 35.04);
    // Herman Miller post to tuning list 2007-05-30 gives
    // TOP period: 	1201.698520
    // TOP generator: 	504.134131
    let tempered_fourth = meantone.pitch_from_steps(&[8, 13]);
    super::assert_between!(504.134, tempered_fourth, 504.135);
    let tempered_octave = meantone.pitch_from_steps(&[19, 31]);
    super::assert_between!(1201.698, tempered_octave, 1201.699);
    // another way of getting the octave
    let tempered_octave = meantone.tuning_map()[0];
    super::assert_between!(1201.698, tempered_octave, 1201.699);
    // another way of getting the fourth
    let tempered_fourth = meantone.pitch_from_primes(&[2, -1, 0]);
    super::assert_between!(504.134, tempered_fourth, 504.135);
}

#[test]
fn meantone_error() {
    let limit5 = super::PrimeLimit::new(5);
    let meantone = make_meantone(&limit5);
    // this gives 1.6985, primerr.pdf says 1.707
    super::assert_between!(1.69, meantone.error(), 1.71)
}

#[test]
fn meantone_optimal_sretch_error() {
    let limit5 = super::PrimeLimit::new(5);
    let meantone = make_meantone(&limit5);
    // same value as above
    super::assert_between!(1.69, meantone.optimal_sretch_error(), 1.71)
}

// Duplicate of TemperamentClass test
#[test]
fn fokker_block() {
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

// Duplicate of TemperamentClass test
#[test]
fn generators() {
    let limit11 = super::PrimeLimit::new(11);
    let marvel = make_marvel(&limit11);
    let twotoe = marvel.generators_from_primes(&vec![3, 0, 0, -1, 0]);
    assert_eq!(twotoe, vec![4, 6, 8]);
}
