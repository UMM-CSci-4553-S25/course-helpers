use ec_core::test_results::{Score, TestResults};
use ec_linear::genome::bitstring::Bitstring;

use crate::processor::Processor;

pub struct PrintMonitor<T>
where
    T: Ord,
{
    best_error_so_far: T,
}

impl Default for PrintMonitor<u64> {
    fn default() -> Self {
        PrintMonitor {
            best_error_so_far: u64::MAX,
        }
    }
}

impl Processor<(usize, i64, u64)> for PrintMonitor<u64> {
    fn process(&mut self, &(sample_number, solution, error): &(usize, i64, u64)) {
        if error < self.best_error_so_far {
            self.best_error_so_far = error;
            println!(
                "New best solution found:  {:25} with error {:25} at sample number {:25}",
                solution, error, sample_number
            );
        }
    }
}

impl Default for PrintMonitor<Score<i64>> {
    fn default() -> Self {
        PrintMonitor {
            best_error_so_far: Score(i64::MIN),
        }
    }
}

impl Processor<(usize, Bitstring, TestResults<Score<i64>>)> for PrintMonitor<Score<i64>> {
    fn process(
        &mut self,
        (sample_number, solution, score): &(usize, Bitstring, TestResults<Score<i64>>),
    ) {
        if score.total_result > self.best_error_so_far {
            self.best_error_so_far = Score(score.total_result.0);
            println!(
                "New best solution found:  {:25} with error {:25} at sample number {:25}",
                solution, score, sample_number
            )
        }
    }
}
