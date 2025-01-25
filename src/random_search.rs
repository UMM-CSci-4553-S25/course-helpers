use std::{marker::PhantomData, sync::Mutex};

use bon::Builder;
use ec_core::individual::scorer::Scorer;
use rand::{prelude::Distribution, rng};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
// use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Debug)]
pub struct RandomSearchError {}

#[derive(Debug, Builder)]
pub struct RandomSearch<Ge, GM, Sc, Scr, Ins>
// You typically wouldn't put all these constraints on the struct itself, instead
// you'd put them on the `impl` block for the struct. But I'm doing it here to
// make the constraints more visible and (hopefully) make some of the error
// messages more helpful for people new to Rust.
where
    Ge: Clone + std::fmt::Debug + Sync + Send,
    GM: Distribution<Ge> + Sync + Send,
    Sc: std::fmt::Debug + Sync + Send,
    Scr: Scorer<Ge, Score = Sc> + Sync + Send,
    // The number of this particular genome, the genome, and its score.
    Ins: FnMut(usize, Ge, Sc) + Sync + Send,
{
    #[builder(default = 1_000)]
    num_to_search: usize,

    #[builder(default = true)]
    parallel_search: bool,

    genome_maker: GM,
    scorer: Scr,
    inspector: Ins,

    // We need this because `RandomSearch` depends on the type `Ge` but doesn't
    // actually contain an instance of it. This is a way to tell Rust that `Ge`
    // is a type that we care about, but we don't actually have an instance of.
    _genome: Option<PhantomData<Ge>>,
}

impl<Ge, GM, Sc, Scr, Ins> RandomSearch<Ge, GM, Sc, Scr, Ins>
where
    Ge: Clone + std::fmt::Debug + Sync + Send,
    GM: Distribution<Ge> + Sync + Send,
    Sc: std::fmt::Debug + Sync + Send,
    Scr: Scorer<Ge, Score = Sc> + Sync + Send,
    // The number of this particular genome, the genome, and its score.
    Ins: FnMut(usize, Ge, Sc) + Sync + Send,
{
    pub fn search(&mut self) -> Result<(), RandomSearchError> {
        if self.parallel_search {
            self.search_parallel()
        } else {
            self.search_sequential()
        }
    }

    fn search_parallel(&mut self) -> Result<(), RandomSearchError> {
        let inspector = Mutex::new(&mut self.inspector);
        (0..self.num_to_search)
            .into_par_iter()
            .for_each(|sample_number| {
                // Generate a random genome as a "solution"
                let sample = self.genome_maker.sample(&mut rng());
                // Score the solution
                let score = self.scorer.score(&sample);
                (inspector.lock().unwrap())(sample_number, sample.clone(), score);
            });

        Ok(())
    }

    fn search_sequential(&mut self) -> Result<(), RandomSearchError> {
        for sample_number in 0..self.num_to_search {
            // Generate a random genome as a "solution"
            let sample = self.genome_maker.sample(&mut rng());
            // Score the solution
            let score = self.scorer.score(&sample);
            (self.inspector)(sample_number, sample.clone(), score);
        }

        Ok(())
    }
}
