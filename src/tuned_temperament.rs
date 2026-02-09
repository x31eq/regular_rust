extern crate nalgebra as na;
use super::temperament_class::TemperamentClass;
use super::{Cents, ETSlice, Tuning, map};
use na::{DMatrix, DVector};

pub trait TunedTemperament: TemperamentClass {
    fn plimit(&self) -> &[Cents];
    fn tuning(&self) -> &Tuning;

    fn tuning_map(&self) -> Tuning {
        let mapping = &self.mapping();
        let rank = mapping.len();
        let dimension = self.plimit().len();
        let tuning = DVector::from_vec(self.tuning().clone());
        let flattened = mapping
            .iter()
            .flat_map(|mapping| mapping.iter().map(|&m| m as f64));
        let melody = DMatrix::from_iterator(dimension, rank, flattened);
        (melody * tuning).iter().cloned().collect()
    }

    fn weighted_tuning_map(&self) -> Tuning {
        self.tuning_map()
            .iter()
            .zip(self.plimit())
            .map(|(&t, &p)| t / p)
            .collect()
    }

    fn mistunings(&self) -> Tuning {
        let tuning_map = self.tuning_map();
        let comparison = tuning_map.iter().zip(self.plimit().iter());
        comparison.map(|(&x, y)| x - y).collect()
    }

    /// Octave or first-harmonic stretch
    fn stretch(&self) -> f64 {
        self.tuning_map()[0] / self.plimit()[0]
    }

    fn unstretched_tuning(&self) -> Tuning {
        map(|x| x / self.stretch(), self.tuning())
    }

    fn unstretched_tuning_map(&self) -> Tuning {
        map(|x| x / self.stretch(), &self.tuning_map())
    }

    fn unstretched_mistunings(&self) -> Tuning {
        let tuning_map = self.unstretched_tuning_map();
        let comparison = tuning_map.iter().zip(self.plimit().iter());
        comparison.map(|(&x, y)| x - y).collect()
    }

    fn pitch_from_steps(&self, interval: &ETSlice) -> Cents {
        self.tuning()
            .iter()
            .zip(interval)
            .map(|(&x, &y)| x * y as Cents)
            .sum()
    }

    fn pitch_from_primes(&self, interval: &ETSlice) -> Cents {
        self.pitch_from_steps(&self.generators_from_primes(interval))
    }
}
