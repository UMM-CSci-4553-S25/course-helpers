use bon::Builder;
use ec_core::{
    distributions::collection::ConvertToCollectionGenerator,
    generation::Generation,
    individual::{
        ec::{EcIndividual, WithScorer},
        scorer::Scorer as IndividualScorer,
    },
    operator::{
        genome_extractor::GenomeExtractor,
        genome_scorer::GenomeScorer,
        mutator::{Mutate, Mutator},
        recombinator::{Recombinator, Recombine},
        selector::{Select, Selector},
        Composable,
    },
};
use ec_linear::genome::bitstring::Bitstring;
use rand::{
    distr::{Bernoulli, Distribution},
    rng,
};
use std::fmt::Debug;

// TODO: What if we want to allow people to specify either a recombinator or a mutator
// or both? (They have to provide at least one, but they don't have to provide both.)

#[derive(Builder)]
pub struct Run<Scorer, Sel, Rec, Mut, Ins>
// You typically wouldn't put all these constraints on the struct itself, instead
// you'd put them on the `impl` block for the struct. But I'm doing it here to
// make the constraints more visible and (hopefully) make some of the error
// messages more helpful for people new to Rust.
where
    Scorer: IndividualScorer<Bitstring> + Send + Sync,
    Scorer::Score: Debug + Send + Sync + Ord,
    // Selector, Recombinator, and Mutator
    Sel: Selector<Vec<EcIndividual<Bitstring, Scorer::Score>>> + Send + Sync,
    Rec: Recombinator<[Bitstring; 2], Output = Bitstring> + Send + Sync,
    Mut: Mutator<Bitstring> + Send + Sync,
    // All associated error types have to implement `std::error::Error`.
    // They also have to be `Send` and `Sync` if we're using parallel evaluation
    // so that errors can propagate across threads.
    // They also have to be bounded by `'static` lifetimes so they can be held to the
    // end of the program as necessary.
    Sel::Error: std::error::Error + Send + Sync + 'static,
    Rec::Error: std::error::Error + Send + Sync + 'static,
    Mut::Error: std::error::Error + Send + Sync + 'static,
    // Inspector
    Ins: FnMut(usize, &Vec<EcIndividual<Bitstring, Scorer::Score>>),
{
    bit_length: usize,

    #[builder(default = 100)]
    population_size: usize,

    #[builder(default = usize::MAX)]
    max_generations: usize,

    #[builder(default = true)]
    parallel_evaluation: bool,

    scorer: Scorer,
    selector: Sel,
    recombinator: Rec,
    mutator: Mut,

    inspector: Ins,
}

#[expect(clippy::match_bool, reason = "I like the `match` instead of `if`")]
impl<Scorer, Sel, Rec, Mut, Ins> Run<Scorer, Sel, Rec, Mut, Ins>
where
    Scorer: IndividualScorer<Bitstring> + Send + Sync,
    Scorer::Score: Debug + Send + Sync + Ord,
    // Selector, Recombinator, and Mutator
    Sel: Selector<Vec<EcIndividual<Bitstring, Scorer::Score>>> + Send + Sync,
    Rec: Recombinator<[Bitstring; 2], Output = Bitstring> + Send + Sync,
    Mut: Mutator<Bitstring> + Send + Sync,
    // All associated error types have to implement `std::error::Error`.
    // They also have to be `Send` and `Sync` if we're using parallel evaluation
    // so that errors can propagate across threads.
    // They also have to be bounded by `'static` lifetimes so they can be held to the
    // end of the program as necessary.
    Sel::Error: std::error::Error + Send + Sync + 'static,
    Rec::Error: std::error::Error + Send + Sync + 'static,
    Mut::Error: std::error::Error + Send + Sync + 'static,
    // Inspector
    Ins: FnMut(usize, &Vec<EcIndividual<Bitstring, Scorer::Score>>),
{
    /// # Errors
    ///
    /// This can return an error if:
    ///    - The argument to the Bernoulli constructor is out of range
    ///    - The population is empty at some point, so `Best::select` fails (this should
    ///      never happen)
    ///    - Creating a new generation fails, probably in creating or scoring new individuals
    pub fn execute(mut self) -> anyhow::Result<Vec<EcIndividual<Bitstring, Scorer::Score>>> {
        let mut rng = rng();

        // Create the initial population for the run
        let population = self.initial_population(&mut rng)?;

        // Make an operator that takes a population and generates a new (child) individual.
        let child_maker =
            // Select a random individual to be a parent
            Select::new(self.selector)
            // Select twice, generating two parents
            .apply_twice()
            // Extract the genomes from those two parents, yielding a pair of genomes (`Bitstring`s)
            .then_map(GenomeExtractor)
            // Combine those genomes (`Bitstrings`s) into a new child genome
            .then(Recombine::new(self.recombinator))
            // Mutate the resulting genome (`Bitstring`)
            .then(Mutate::new(self.mutator))
            // Score the resulting mutated genome to generate an `Individual`
            .wrap::<GenomeScorer<_, _>>(&self.scorer);

        let mut generation = Generation::new(child_maker, population);

        for generation_number in 0..self.max_generations {
            (self.inspector)(generation_number, generation.population());
            match self.parallel_evaluation {
                true => generation.par_next()?,
                false => generation.serial_next()?,
            }
        }

        (self.inspector)(self.max_generations, generation.population());

        Ok(generation.into_population())
    }

    fn initial_population(
        &self,
        rng: &mut rand::prelude::ThreadRng,
    ) -> anyhow::Result<Vec<EcIndividual<Bitstring, Scorer::Score>>> {
        let population =
            // `Bernoulli` can be used to generate random booleans with the
            // given probability of bits being `true` A small probability
            // creates initial bitstrings that are mostly `false`.
            // Should become part of command line arguments.
            Bernoulli::new(0.01)?
            // Generate a `Bitstring` of length `self.bit_length`
            .into_collection_generator(self.bit_length)
            // Adds a scorer to the `Bitstring`, creating an `Individual`
            .with_scorer(&self.scorer)
            // Create a `Population` of `self.population_size` `Individual`s
            .into_collection_generator(self.population_size)
            // Actually sample the distribution to get the initial population.
            .sample(rng);
        Ok(population)
    }
}
