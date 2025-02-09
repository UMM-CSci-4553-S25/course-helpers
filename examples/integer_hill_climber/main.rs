use rand::distr::{uniform, Distribution, StandardUniform, Uniform};
use std::convert::Infallible;

use course_helpers::{hill_climber::HillClimber, inspector::update_best};
use ec_core::{individual::scorer::FnScorer, operator::mutator::Mutator, test_results::Error};

struct IntegerMutator {
    distribution: Uniform<i32>,
}

impl IntegerMutator {
    pub fn new(max_step: i32) -> Result<Self, uniform::Error> {
        let distribution = Uniform::new(-max_step, max_step)?;
        Ok(Self { distribution })
    }
}

impl Mutator<i32> for IntegerMutator {
    type Error = Infallible;

    fn mutate<R: rand::Rng + ?Sized>(&self, genome: i32, rng: &mut R) -> Result<i32, Self::Error> {
        Ok(genome.saturating_add(self.distribution.sample(rng)))
    }
}

fn main() -> anyhow::Result<()> {
    let num_to_create = 1_000_000;
    let target: i32 = 589;
    let scorer = FnScorer(|value: &i32| Error(value.abs_diff(target)));

    // Create a `Distribution` that generates `i64`s when sampled
    let genome_maker = StandardUniform;

    let mut best = None;

    let mut hill_climber = HillClimber::builder()
        .num_to_search(num_to_create)
        .num_children_per_step(10)
        .always_replace(false)
        .genome_maker(genome_maker)
        .mutator(IntegerMutator::new(100_000)?)
        .scorer(scorer)
        .inspector(|solution_chunk| update_best(&mut best, solution_chunk))
        .build();

    hill_climber.search()?;

    Ok(())
}
