use std::hash::Hash;

use crate::{
    error::{CountError, CountResult},
    ElementSet,
};

#[derive(Debug)]
pub struct StreamCountEstimator<T> {
    elements: ElementSet<T>,
    tresh: usize,
    round_prob: f64,
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
    T: Hash + Eq + Sized,
{
    pub fn new(epsilon: f64, delta: f64, stream_length: usize) -> CountResult<Self> {
        in_unit_interval(epsilon)?;
        in_unit_interval(delta)?;
        let tresh = (12.0 / epsilon.powi(2) * (8.0 * (stream_length as f64) / delta).log2()).ceil()
            as usize;
        Ok(StreamCountEstimator {
            elements: ElementSet::default(),
            tresh,
            round_prob: 1.0,
        })
    }

    fn process_element(&mut self, element: &T) {
        if self.elements.contains(element) {
            self.elements.remove(element);
        }
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
