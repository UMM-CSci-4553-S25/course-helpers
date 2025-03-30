use ec_core::{
    individual::scorer::Scorer,
    test_results::{self, TestResults},
};
use ordered_float::OrderedFloat;
use push::genome::plushy::Plushy;
use rand::Rng;

use super::Simplifier;

pub struct DropOne<S> {
    scorer: S,
    num_simplification_attempts: usize,
    acceptable_single_error_difference: f64,
}

impl<S> DropOne<S> {
    pub fn new(
        scorer: S,
        num_simplification_attempts: usize,
        acceptable_single_error_difference: f64,
    ) -> Self {
        Self {
            scorer,
            num_simplification_attempts,
            acceptable_single_error_difference,
        }
    }

    fn drop_random_instruction<R: Rng>(&self, genome: &Plushy, rng: &mut R) -> Plushy {
        let mut genes = genome.get_genes();
        let index = rng.random_range(0..genes.len());
        let _ = genes.remove(index);
        Plushy::new(genes)
    }

    // TODO: We could change this allow _improvements_, i.e., reductions in
    //   error as the result of simplication. That can happen, and this would
    //   reject anything like that.
    fn nearly_equal_scores(
        &self,
        first: &TestResults<test_results::Error<OrderedFloat<f64>>>,
        second: &TestResults<test_results::Error<OrderedFloat<f64>>>,
    ) -> bool {
        for (x, y) in first.results.iter().zip(second.results.iter()) {
            if (x.0 - y.0).abs() > self.acceptable_single_error_difference {
                return false;
            }
        }
        true
    }
}

impl<S> Simplifier<Plushy> for DropOne<S>
where
    S: Scorer<Plushy, Score = TestResults<test_results::Error<OrderedFloat<f64>>>>,
{
    fn simplify_genome<R: Rng>(&self, mut genome: Plushy, rng: &mut R) -> Plushy {
        let original_score = self.scorer.score(&genome);
        for _ in 0..self.num_simplification_attempts {
            let possible_simplification = self.drop_random_instruction(&genome, rng);
            let new_score = self.scorer.score(&possible_simplification);
            if self.nearly_equal_scores(&original_score, &new_score) {
                genome = possible_simplification;
            }
        }
        genome
    }
}
