use ec_core::test_results::{Score, TestResults};
use ec_linear::genome::bitstring::Bitstring;

use crate::processor::Processor;

pub struct MinMaxDistance<G> {
    min_error_individual: Option<G>,
    min_error: u64,
    max_error_individual: Option<G>,
    max_error: u64,
}

impl<G> Default for MinMaxDistance<G> {
    fn default() -> Self {
        Self {
            min_error_individual: None,
            min_error: u64::MAX,
            max_error_individual: None,
            max_error: u64::MIN,
        }
    }
}

impl Processor<(usize, i64, u64)> for MinMaxDistance<i64> {
    fn process(&mut self, &(_, individual, error): &(usize, i64, u64)) {
        if error < self.min_error {
            self.min_error_individual = Some(individual);
            self.min_error = error;
            // println!(
            //     "New best solution found:  {:25} with error {:25}",
            //     individual, error
            // );
        }
        if error > self.max_error {
            self.max_error_individual = Some(individual);
            self.max_error = error;
            // println!(
            //     "New worst solution found: {:25} with error {:25}",
            //     individual, error
            // );
        }
    }

    fn finalize_and_print(&self) {
        println!("The farthest distance was {:25}", self.max_error);
        println!("The nearest distance was  {:25}", self.min_error);
    }
}

impl Processor<(usize, Bitstring, TestResults<Score<i64>>)> for MinMaxDistance<Bitstring> {
    fn process(&mut self, (_, individual, error): &(usize, Bitstring, TestResults<Score<i64>>)) {
        if (error.total_result.0 as u64) < self.min_error {
            self.min_error_individual = Some(individual.clone());
            self.min_error = error.total_result.0 as u64;
        }
        if (error.total_result.0 as u64) > self.max_error {
            self.max_error_individual = Some(individual.clone());
            self.max_error = error.total_result.0 as u64;
        }
    }

    fn finalize_and_print(&self) {
        println!("The farthest distance was {:25}", self.max_error);
        println!("The nearest distance was  {:25}", self.min_error);
    }
}
