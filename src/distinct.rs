use rand::distributions::{Bernoulli, Distribution};
use std::hash::Hash;

use crate::{
    error::{CountError, CountResult},
    ElementSet,
};

#[derive(Debug)]
pub struct StreamCountEstimator<T> {
    elements: ElementSet<T>,
    tresh: usize,
    sampling_round: u32,
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

impl<T> StreamCountEstimator<T>
where
    T: Hash + Eq + Sized + Clone,
{
    pub fn new(epsilon: f64, delta: f64, stream_length: usize) -> CountResult<Self> {
        in_unit_interval(epsilon)?;
        in_unit_interval(delta)?;
        let tresh = (12.0 / epsilon.powi(2) * (8.0 * (stream_length as f64) / delta).log2()).ceil()
            as usize;
        Ok(StreamCountEstimator {
            elements: ElementSet::with_capacity(tresh),
            tresh,
            sampling_round: 1,
        })
    }

    fn process_element(&mut self, element: T) -> CountResult<()> {
        let prob_dist = Bernoulli::from_ratio(1, self.sampling_round)
            .map_err(|err| CountError::Message(err.to_string()))?;
        if prob_dist.sample(&mut rand::thread_rng()) {
            self.elements.insert(element);
        } else if self.elements.contains(&element) {
            self.elements.remove(&element);
        }
        if self.elements.len() == self.tresh {
            let mut updatet_elements = ElementSet::<T>::with_capacity(self.tresh);

            let prob_dist =
                Bernoulli::from_ratio(1, 2).map_err(|err| CountError::Message(err.to_string()))?;
            let mut randomness = rand::thread_rng();
            for elem in self.elements.iter() {
                if prob_dist.sample(&mut randomness) {
                    updatet_elements.insert(elem.clone());
                }
            }
            self.elements = updatet_elements;
            self.sampling_round *= 2;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use insta::*;

    use super::StreamCountEstimator;

    #[test]
    fn incorrect_input_params() {
        let err_epsilon =
            StreamCountEstimator::<u32>::new(-1.0, 0.5, 1).expect_err("Expected error.");
        assert_snapshot!(err_epsilon, @"CountError(WrongInitializiation(Input -1 is negative.))");
        let err_delta = StreamCountEstimator::<u32>::new(1.0, 1.5, 1).expect_err("Expected error.");
        assert_snapshot!(err_delta, @"CountError(WrongInitializiation(Input 1.5 is larger than 1.))");
    }
}
