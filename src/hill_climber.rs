use core::slice;
use std::marker::PhantomData;

use bon::Builder;
use ec_core::{individual::scorer::Scorer, operator::mutator::Mutator};
use itertools::Itertools;
use rand::{prelude::Distribution, rng};

#[derive(Debug, thiserror::Error)]
pub enum HillClimberError<MutationError> {
    Mutation(#[from] MutationError),
    ZeroSizedChunk,
}

#[derive(Debug, Builder)]
pub struct HillClimber<Ge, GM, Mut, Sc, Scr, Ins>
// You typically wouldn't put all these constraints on the struct itself, instead
// you'd put them on the `impl` block for the struct. But I'm doing it here to
// make the constraints more visible and (hopefully) make some of the error
// messages more helpful for people new to Rust.
where
    Ge: Clone + std::fmt::Debug + Sync + Send,
    GM: Distribution<Ge> + Sync + Send,
    Mut: Mutator<Ge> + Sync + Send,
    Sc: Ord + PartialOrd + std::fmt::Debug + Sync + Send,
    Scr: Scorer<Ge, Score = Sc> + Sync + Send,
    // The number of this particular genome, the genome, and its score.
    Ins: FnMut(&[(usize, Ge, Sc)]) + Sync + Send,
{
    // We need `PhantomData` because `RandomSearch` depends on the type `Ge` but doesn't
    // actually contain an instance of it. This is a way to tell Rust that `Ge`
    // is a type that we care about, but we don't actually have an instance of.
    // The `builder(field)` attribute tells the `Builder` derive macro that this
    // is a field that is _not_ specified in the build process.
    #[builder(field)]
    _p: PhantomData<Ge>,

    #[builder(default = 1_000)]
    num_to_search: usize,

    #[builder(default = 1)]
    num_children_per_step: usize,

    /// Do we _always_ replace the current solution with the best of the "child"
    /// solutions, even if they aren't better than the current solution?
    #[builder(default = false)]
    always_replace: bool,

    genome_maker: GM,
    mutator: Mut,
    scorer: Scr,
    inspector: Ins,
}

impl<Ge, GM, Mut, Sc, Scr, Ins> HillClimber<Ge, GM, Mut, Sc, Scr, Ins>
where
    Ge: Clone + std::fmt::Debug + Sync + Send,
    GM: Distribution<Ge> + Sync + Send,
    Mut: Mutator<Ge> + Sync + Send,
    Sc: Ord + PartialOrd + std::fmt::Debug + Sync + Send + Clone,
    Scr: Scorer<Ge, Score = Sc> + Sync + Send,
    // The number of this particular genome, the genome, and its score.
    Ins: FnMut(&[(usize, Ge, Sc)]) + Sync + Send,
{
    pub fn search(&mut self) -> Result<(), HillClimberError<Mut::Error>> {
        let initial_candidate = self.genome_maker.sample(&mut rng());
        self.search_sequential(initial_candidate)
    }

    fn search_sequential(
        &mut self,
        initial_candidate: Ge,
    ) -> Result<(), HillClimberError<Mut::Error>> {
        let mut rng = rand::rng();

        let initial_score = self.scorer.score(&initial_candidate);
        let mut current_scored_best = (0, initial_candidate, initial_score);

        (self.inspector)(slice::from_ref(&current_scored_best));

        for indices in &(1..self.num_to_search).chunks(self.num_children_per_step) {
            let best_in_chunk = indices
                .map(|sample_number| -> Result<_, HillClimberError<Mut::Error>> {
                    let child = self
                        .mutator
                        .mutate(current_scored_best.1.clone(), &mut rng)?;
                    let score = self.scorer.score(&child);
                    Ok((sample_number, child, score))
                })
                .process_results(|iter| {
                    iter.max_by(|(_, _, first_score), (_, _, second_score)| {
                        first_score.cmp(second_score)
                    })
                })?
                .ok_or(HillClimberError::ZeroSizedChunk)?;

            if self.always_replace || best_in_chunk.2 > current_scored_best.2 {
                current_scored_best = best_in_chunk;
                (self.inspector)(slice::from_ref(&current_scored_best));
            }
        }

        Ok(())
    }
}
