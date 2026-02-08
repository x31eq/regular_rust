use super::cangwu::TenneyWeighted;
use super::temperament_class::TemperamentClass;
use super::{Cents, Mapping, Tuning};

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
