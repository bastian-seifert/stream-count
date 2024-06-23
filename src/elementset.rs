use std::{collections::HashSet, hash::Hash};

/// The ElementSet trait defines the operations needed
/// for the stream count algorithm to operate (a subset
/// of the operations availabel on hash sets).
///
/// This trait allows flexibility for the user (e.g., choose
/// whatever data structure fits your use case best) as well
/// as enabling snapshot testing (as hashsets are non-deterministic).
pub trait ElementSet {
    type Element;
    fn with_capacity(capacity: usize) -> Self
    where
        Self: Sized;
    /// Inserts `elem` if it is not in `self`,
    /// otherwise does nothing.
    fn insert(&mut self, elem: Self::Element);
    fn contains(&self, elem: &Self::Element) -> bool;
    fn remove(&mut self, elem: &Self::Element);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn iter(&self) -> impl Iterator<Item = &Self::Element>;
}

impl<T> ElementSet for HashSet<T>
where
    T: Eq + Hash,
{
    type Element = T;

    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }

    fn insert(&mut self, elem: Self::Element) {
        HashSet::insert(self, elem);
    }

    fn contains(&self, elem: &Self::Element) -> bool {
        HashSet::contains(self, elem)
    }

    fn remove(&mut self, elem: &Self::Element) {
        HashSet::remove(self, elem);
    }

    fn len(&self) -> usize {
        HashSet::len(self)
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        HashSet::iter(self)
    }
}

/// This implementation is very inefficient and should
/// only used for snapshot testing.
impl<T> ElementSet for Vec<T>
where
    T: Eq,
{
    type Element = T;

    fn with_capacity(capacity: usize) -> Self {
        Vec::with_capacity(capacity)
    }

    fn insert(&mut self, elem: Self::Element) {
        if !self.contains(&elem) {
            self.push(elem);
        }
    }

    fn contains(&self, elem: &Self::Element) -> bool {
        self.as_slice().iter().any(|val| val == elem)
    }

    fn remove(&mut self, elem: &Self::Element) {
        let Some(pos) = self.as_slice().iter().position(|val| val == elem) else {
            return;
        };
        Vec::remove(self, pos);
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        self.as_slice().iter()
    }
}

#[cfg(test)]
mod test {
    use std::marker::PhantomData;
    use std::{fmt::Debug, time::Duration};

    use async_trait::async_trait;
    use proptest::prelude::*;
    use proptest_stateful::{ModelState, ProptestStatefulConfig};

    use super::ElementSet;

    #[derive(Clone, Debug)]
    enum ElementSetOps {
        Insert(u32),
        Remove(u32),
    }
    #[derive(Clone, Debug, Default)]
    struct TestState<E: ElementSet + Clone + Default> {
        marker: PhantomData<E>,
    }

    #[async_trait(?Send)]
    impl<E> ModelState for TestState<E>
    where
        E: ElementSet<Element = u32> + Clone + Default + Debug,
    {
        type Operation = ElementSetOps;
        type RunContext = E;
        type OperationStrategy = BoxedStrategy<Self::Operation>;

        fn op_generators(&self) -> Vec<Self::OperationStrategy> {
            // For each step test, arbitrarily pick Insert or Remove, regardless of the test state:
            // TODO: Add strategy to gen numbers
            vec![
                Just(ElementSetOps::Insert(0)).boxed(),
                Just(ElementSetOps::Remove(0)).boxed(),
            ]
        }

        // No preconditions to worry about or test state to maintain yet
        fn preconditions_met(&self, _op: &Self::Operation) -> bool {
            true
        }
        fn next_state(&mut self, _op: &Self::Operation) {}

        async fn init_test_run(&self) -> Self::RunContext {
            E::with_capacity(100)
        }

        async fn run_op(&self, op: &Self::Operation, ctxt: &mut Self::RunContext) {
            match op {
                ElementSetOps::Insert(val) => ctxt.insert(*val),
                ElementSetOps::Remove(val) => ctxt.remove(val),
            }
        }

        async fn check_postconditions(&self, _ctxt: &mut Self::RunContext) {}
        async fn clean_up_test_run(&self, _ctxt: &mut Self::RunContext) {}
    }

    #[test]
    fn run_cases_vec() {
        let config = ProptestStatefulConfig {
            min_ops: 10,
            max_ops: 50,
            test_case_timeout: Duration::from_secs(60),
            proptest_config: ProptestConfig::default(),
        };

        proptest_stateful::test::<TestState<Vec<u32>>>(config);
    }
}
