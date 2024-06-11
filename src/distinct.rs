use rand::{
    distributions::{Bernoulli, Distribution},
    Rng,
};
use std::fmt::Debug;
use std::hash::Hash;

use crate::{
    error::{CountError, CountResult},
    ElementSet,
};

#[derive(Debug)]
pub struct StreamCountEstimator<T> {
    elements: ElementSet<T>,
    capacity: usize,
    sampling_round: f64,
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
    T: Hash + Eq + Sized + Clone + Debug,
{
    pub fn new(epsilon: f64, delta: f64, stream_length: usize) -> CountResult<Self> {
        in_unit_interval(epsilon)?;
        in_unit_interval(delta)?;
        let capacity = (12.0 / epsilon.powi(2) * (8.0 * (stream_length as f64) / delta).log2())
            .ceil() as usize;
        Ok(StreamCountEstimator {
            elements: ElementSet::with_capacity(capacity),
            capacity,
            sampling_round: 1.0,
        })
    }

    pub fn with_capacity(capacity: usize) -> CountResult<Self> {
        Ok(StreamCountEstimator {
            elements: ElementSet::with_capacity(capacity),
            capacity,
            sampling_round: 1.0,
        })
    }

    fn process_element(&mut self, element: T) -> CountResult<()> {
        self.process_element_with_randomness(element, &mut rand::thread_rng())
    }

    fn process_element_with_randomness<R: Rng + ?Sized>(
        &mut self,
        element: T,
        randomness: &mut R,
    ) -> CountResult<()> {
        let prob_dist = Bernoulli::from_ratio(1, self.sampling_round as u32)
            .map_err(|err| CountError::Message(err.to_string()))?;
        if prob_dist.sample(randomness) {
            self.elements.insert(element);
        } else if self.elements.contains(&element) {
            self.elements.remove(&element);
        }
        if self.elements.len() == self.capacity - 1 {
            let mut updatet_elements = ElementSet::<T>::with_capacity(self.capacity);

            let prob_dist =
                Bernoulli::from_ratio(1, 2).map_err(|err| CountError::Message(err.to_string()))?;
            for elem in self.elements.iter() {
                if prob_dist.sample(randomness) {
                    updatet_elements.insert(elem.clone());
                }
            }
            self.elements = updatet_elements;
            self.sampling_round *= 2.0;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{
        borrow::{Borrow, BorrowMut},
        cell::RefCell,
    };

    use insta::*;
    use rand::{
        distributions::{Bernoulli, Distribution},
        rngs::StdRng,
        SeedableRng,
    };
    use rand_chacha::ChaCha8Rng;
    use rand_pcg::Pcg64;

    use super::StreamCountEstimator;

    thread_local!(
    pub static RNG: RefCell<Pcg64> = RefCell::new(Pcg64::seed_from_u64(
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    )));

    #[test]
    fn random_bool() {
        let mut randomness = ChaCha8Rng::seed_from_u64(1337);

        let prob_dist = Bernoulli::from_ratio(1, 2).unwrap();
        let some_other_dist = Bernoulli::from_ratio(1, 4).unwrap();
        let mut bool_vec = Vec::with_capacity(10);
        for _ in 0..10 {
            bool_vec.push(prob_dist.sample(&mut randomness));
            some_other_dist.sample(&mut randomness);
        }
        println!("{bool_vec:?}");
    }

    #[test]
    fn incorrect_input_params() {
        let err_epsilon =
            StreamCountEstimator::<u32>::new(-1.0, 0.5, 1).expect_err("Expected error.");
        assert_snapshot!(err_epsilon, @"CountError(WrongInitializiation(Input -1 is negative.))");
        let err_delta = StreamCountEstimator::<u32>::new(1.0, 1.5, 1).expect_err("Expected error.");
        assert_snapshot!(err_delta, @"CountError(WrongInitializiation(Input 1.5 is larger than 1.))");
    }

    #[test]
    fn process_element() {
        let mut scount = StreamCountEstimator::<usize>::with_capacity(10).unwrap();

        let mut randomness = StdRng::seed_from_u64(1);
        for num in 0..100 {
            scount
                .process_element_with_randomness(num, &mut randomness)
                .unwrap();
        }
        assert_debug_snapshot!(scount, @r###"
        StreamCountEstimator {
            elements: {
                41,
                66,
                1,
                32,
                37,
                91,
            },
            capacity: 10,
            sampling_round: 32.0,
        }
        "###);
    }
}
