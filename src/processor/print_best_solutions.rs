use ec_core::test_results::{Score, TestResults};
use ec_linear::genome::bitstring::Bitstring;

use crate::processor::Processor;

pub struct PrintBestSolution<T>
where
    T: Ord,
{
    best_error_so_far: Option<T>,
}

impl<T> Default for PrintBestSolution<T>
where
    T: Ord,
{
    fn default() -> Self {
        PrintBestSolution {
            best_error_so_far: None,
        }
    }
}

impl Processor<(usize, i64, u64)> for PrintBestSolution<u64> {
    fn process(&mut self, &(sample_number, solution, error): &(usize, i64, u64)) {
        if self
            .best_error_so_far
            .is_none_or(|best_error_so_far| error < best_error_so_far)
        {
            self.best_error_so_far = Some(error);
            println!(
                "New best solution found:  {:25} with error {:25} at sample number {:25}",
                solution, error, sample_number
            );
        }
    }
}

impl Processor<(usize, Bitstring, TestResults<Score<u64>>)> for PrintBestSolution<Score<u64>> {
    fn process(
        &mut self,
        (sample_number, solution, score): &(usize, Bitstring, TestResults<Score<u64>>),
    ) {
        if self
            .best_error_so_far
            .as_ref()
            .is_none_or(|best_error_so_far| *best_error_so_far < score.total_result)
        {
            self.best_error_so_far = Some(Score(score.total_result.0));
            println!(
                "New best solution found:  {:25} with error {:25} at sample number {:25}",
                solution, score, sample_number
            );
        }
    }
}
