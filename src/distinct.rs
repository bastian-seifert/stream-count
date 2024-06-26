use rand::{
    distributions::{Bernoulli, Distribution},
    Rng,
};
use std::fmt::Debug;

use crate::{
    elementset::*,
    error::{CountError, CountResult},
};

#[derive(Debug)]
pub struct StreamCountEstimator<E: ElementSet> {
    elements: E,
    capacity: usize,
    sampling_round: usize,
}

/// Throws an error if input is not in [0,1]
fn in_unit_interval(input: f64) -> CountResult<()> {
    if input < 0.0 {
        return Err(CountError::WrongInitialization(format!(
            "Input {input} is negative."
        )));
    }
    if input > 1.0 {
        return Err(CountError::WrongInitialization(format!(
            "Input {input} is larger than 1."
        )));
    }
    Ok(())
}

impl<E> StreamCountEstimator<E>
where
    E: ElementSet,
    E::Element: Clone,
{
    /// Creates a new StreamCountEstimator, giving an `epsilon`-`delta` approximation
    /// for a data stream of length `stream_length`.
    /// The internal space capacity is calculated to guarantee the approximation goodness.
    pub fn new(epsilon: f64, delta: f64, stream_length: usize) -> CountResult<Self> {
        in_unit_interval(epsilon)?;
        in_unit_interval(delta)?;
        let capacity = (12.0 / epsilon.powi(2) * (8.0 * (stream_length as f64) / delta).log2())
            .ceil() as usize;
        Ok(StreamCountEstimator {
            elements: ElementSet::with_capacity(capacity),
            capacity,
            sampling_round: 1,
        })
    }

    /// Creates a new StreamCountEstimator with given space capacity.
    /// The approximation goodness depends on the capacity.
    pub fn with_capacity(capacity: usize) -> CountResult<Self> {
        Ok(StreamCountEstimator {
            elements: ElementSet::with_capacity(capacity),
            capacity,
            sampling_round: 1,
        })
    }

    pub fn estimate_distinct_elements_iter(
        &mut self,
        it: impl Iterator<Item = E::Element>,
    ) -> CountResult<usize> {
        for elem in it.into_iter() {
            self.process_element(elem)?;
        }
        Ok(self.elements.len() * self.sampling_round)
    }

    pub fn estimate_distinct_elements_iter_with_rng<R: Rng + ?Sized>(
        &mut self,
        it: impl Iterator<Item = E::Element>,
        rng: &mut R,
    ) -> CountResult<usize> {
        for elem in it.into_iter() {
            while self.process_element_with_rng(elem.clone(), rng)?.is_none() {}
        }
        Ok(self.elements.len() * self.sampling_round)
    }

    fn process_element(&mut self, element: E::Element) -> CountResult<Option<()>> {
        self.process_element_with_rng(element, &mut rand::thread_rng())
    }

    fn process_element_with_rng<R: Rng + ?Sized>(
        &mut self,
        element: E::Element,
        rng: &mut R,
    ) -> CountResult<Option<()>> {
        let prob_dist = Bernoulli::from_ratio(1, self.sampling_round as u32).map_err(|err| {
            CountError::Message(format!(
                "Could not create probability distribution within sampling round {}: \n {err}",
                self.sampling_round
            ))
        })?;
        if prob_dist.sample(rng) {
            self.elements.insert(element);
        } else if self.elements.contains(&element) {
            self.elements.remove(&element);
        }
        if self.elements.len() == self.capacity {
            let mut updatet_elements = E::with_capacity(self.capacity);

            let prob_dist =
                Bernoulli::from_ratio(1, 2).map_err(|err| CountError::Message(err.to_string()))?;
            for elem in self.elements.iter() {
                if prob_dist.sample(rng) {
                    updatet_elements.insert(elem.clone());
                }
            }
            if updatet_elements.len() == self.capacity {
                return Ok(None);
            }
            self.elements = updatet_elements;
            self.sampling_round *= 2;
        }
        Ok(Some(()))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use insta::*;
    use itertools::Itertools;
    use rand::{rngs::StdRng, Rng, SeedableRng};

    use super::StreamCountEstimator;

    #[test]
    fn incorrect_input_params() {
        let err_epsilon =
            StreamCountEstimator::<HashSet<u32>>::new(-1.0, 0.5, 1).expect_err("Expected error.");
        assert_snapshot!(err_epsilon, @r###"
        CountError(
        	WrongInitializiation(Input -1 is negative.)
        )
        "###);
        let err_delta =
            StreamCountEstimator::<HashSet<u32>>::new(1.0, 1.5, 1).expect_err("Expected error.");
        assert_snapshot!(err_delta, @r###"
        CountError(
        	WrongInitializiation(Input 1.5 is larger than 1.)
        )
        "###);
    }

    #[test]
    fn process_element() {
        let mut scount = StreamCountEstimator::<Vec<usize>>::with_capacity(10).unwrap();

        let mut rng = StdRng::seed_from_u64(1);
        for num in 0..100 {
            scount.process_element_with_rng(num, &mut rng).unwrap();
        }
        assert_debug_snapshot!(scount, @r###"
        StreamCountEstimator {
            elements: [
                6,
                21,
                32,
                35,
                72,
                82,
                88,
            ],
            capacity: 10,
            sampling_round: 16,
        }
        "###);
    }

    #[test]
    fn simple_data_iter_count() {
        let mut rng = StdRng::seed_from_u64(1);
        let input_vec = (0..1000).map(|_| rng.gen_range(0..15)).collect_vec();
        let mut scount = StreamCountEstimator::<Vec<i32>>::with_capacity(10).unwrap();
        let count = scount
            .estimate_distinct_elements_iter_with_rng(input_vec.into_iter(), &mut rng)
            .unwrap();

        assert_eq!(count, 12);
    }

    #[test]
    fn two_elements_full_capacity_long_data_iter() {
        let mut rng = StdRng::seed_from_u64(1337);
        let input_vec = (0..1000000).map(|_| rng.gen_range(0..2)).collect_vec();
        let mut scount = StreamCountEstimator::<Vec<i32>>::with_capacity(3).unwrap();
        let count = scount
            .estimate_distinct_elements_iter_with_rng(input_vec.into_iter(), &mut rng)
            .unwrap();

        assert_debug_snapshot!(scount, @r###"
        StreamCountEstimator {
            elements: [
                1,
                0,
            ],
            capacity: 3,
            sampling_round: 1,
        }
        "###);

        assert_eq!(count, 2);
    }
}
