use microlp::{ComparisonOp, OptimizationDirection, Problem};

use super::cangwu::TenneyWeighted;
use super::temperament_class::TemperamentClass;
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

#[test]
fn meantone() {
    let limit5 = super::PrimeLimit::new(5);
    let meantone_vector = vec![vec![19, 30, 44], vec![31, 49, 72]];
    let mut meantone = TOPTemperament::new(&limit5.pitches, &meantone_vector);
    meantone.optimize();
    assert_eq!(meantone.tuning.len(), 2);
    super::assert_between!(6.07, meantone.tuning[0], 6.08);
    super::assert_between!(35.03, meantone.tuning[1], 35.04);
    // Check that the octave looks like an octave
    let tempered_octave =
        meantone.tuning[0] * 19.0 + meantone.tuning[1] * 31.0;
    super::assert_between!(1155.0, tempered_octave, 1205.0);
}
