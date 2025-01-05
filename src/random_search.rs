use ec_core::individual::scorer::Scorer;
use rand::{prelude::Distribution, rng};
// use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Debug)]
pub struct RandomSearchError {}

pub struct RandomSearch<Ge, GM, Score, Scorer> {
    num_to_search: usize,
    genome_maker: GM,
    scorer: Scorer,
    sender: Option<flume::Sender<(usize, Ge, Score)>>,
    phantom: std::marker::PhantomData<Ge>,
}

impl<Ge, GM, Sc, Scr> RandomSearch<Ge, GM, Sc, Scr>
where
    Ge: Clone + std::fmt::Debug + Sync + Send,
    GM: Distribution<Ge> + Sync + Send,
    Sc: std::fmt::Debug + Sync + Send,
    Scr: Scorer<Ge, Score = Sc> + Sync + Send,
{
    pub fn new(
        num_to_create: usize,
        genome_maker: GM,
        scorer: Scr,
        sender: flume::Sender<(usize, Ge, Sc)>,
    ) -> Self {
        Self {
            num_to_search: num_to_create,
            genome_maker,
            scorer,
            sender: Some(sender),
            phantom: std::marker::PhantomData,
        }
    }

    pub fn run_to_end(&mut self) -> Result<(), RandomSearchError> {
        // Document why we need this whole `.take()` business.
        if let Some(sender) = self.sender.take() {
            // (0..self.num_to_search)
            //     .into_par_iter()
            //     .for_each(|sample_number| {
            //         // Generate a random genome as a "solution"
            //         let sample = self.genome_maker.sample(&mut rng());
            //         // Score the solution
            //         let score = self.scorer.score(&sample);
            //         // Send the solution and score to the channel
            //         sender.send((sample_number, sample, score)).unwrap();
            //     });

            for sample_number in 0..self.num_to_search {
                // Generate a random genome as a "solution"
                let sample = self.genome_maker.sample(&mut rng());
                // Score the solution
                let score = self.scorer.score(&sample);
                sender.send((sample_number, sample, score)).unwrap();
            }
        }

        Ok(())
    }
}
