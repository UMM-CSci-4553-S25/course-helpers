use std::ops::Sub;

use ec_core::{
    individual::scorer::Scorer,
    population::Population,
    test_results::{self, TestResults},
};
use num_traits::Signed;
use push::genome::plushy::Plushy;
use rand::Rng;

use super::Simplifier;

pub struct DropOne<S, Score>
where
    Score: Eq + Ord,
{
    scorer: S,
    num_simplification_attempts: usize,
    acceptable_single_error_difference: Score,
}

impl<S, Score> DropOne<S, Score>
where
    Score: Eq + Ord + Sub + Signed + Copy,
{
    pub fn new(
        scorer: S,
        num_simplification_attempts: usize,
        acceptable_single_error_difference: Score,
    ) -> Self {
        Self {
            scorer,
            num_simplification_attempts,
            acceptable_single_error_difference,
        }
    }

    // TODO: Should this move to `Plushy`?
    fn drop_random_instruction<R: Rng>(&self, genome: &Plushy, rng: &mut R) -> Plushy {
        let mut genes = genome.get_genes();
        // This panics if the range is empty, i.e., if `genes.len()` is 0.
        // We might want to have this return a `Result` type.
        let index = rng.random_range(0..genes.len());
        let _ = genes.remove(index);
        Plushy::new(genes)
    }

    // TODO: We could change this allow _improvements_, i.e., reductions in
    //   error as the result of simplification. That can happen, and this would
    //   reject anything like that.
    // TODO: Should this move to `TestResults`?
    fn nearly_equal_scores(
        &self,
        first: &TestResults<test_results::Error<Score>>,
        second: &TestResults<test_results::Error<Score>>,
    ) -> bool {
        for (x, y) in first.results.iter().zip(second.results.iter()) {
            if (x.0 - y.0).abs() > self.acceptable_single_error_difference {
                return false;
            }
        }
        true
    }
}

impl<S, Score> Simplifier<Plushy> for DropOne<S, Score>
where
    S: Scorer<Plushy, Score = TestResults<test_results::Error<Score>>>,
    Score: Eq + Ord + Sub + Signed + Copy,
{
    fn simplify_genome<R: Rng>(&self, mut genome: Plushy, rng: &mut R) -> Plushy {
        let original_score = self.scorer.score(&genome);
        for _ in 0..self.num_simplification_attempts {
            if genome.get_genes().size() == 0 {
                break;
            }
            let possible_simplification = self.drop_random_instruction(&genome, rng);
            let new_score = self.scorer.score(&possible_simplification);
            if self.nearly_equal_scores(&original_score, &new_score) {
                genome = possible_simplification;
            }
        }
        genome
    }
}
