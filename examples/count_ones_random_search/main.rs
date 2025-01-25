use std::fmt::Display;

use course_helpers::random_search::{RandomSearch, RandomSearchError};
use ec_core::{
    distributions::collection::ConvertToCollectionGenerator,
    individual::scorer::FnScorer,
    test_results::{Score, TestResults},
};
use ec_linear::genome::bitstring::Bitstring;
use rand::distr::StandardUniform;

#[must_use]
pub fn count_ones(bits: &[bool]) -> TestResults<Score<u64>> {
    bits.iter().copied().map(u64::from).collect()
}

fn main() -> Result<(), RandomSearchError> {
    let num_to_create = 1_000_000;

    let num_bits = 10;

    let scorer = FnScorer(|bitstring: &Bitstring| count_ones(&bitstring.bits));

    // Create a `Distribution` that generates `Bitstring`s when sampled
    let genome_maker = StandardUniform.into_collection_generator(num_bits);

    let mut best = None;

    let mut random_search = RandomSearch::builder()
        .num_to_search(num_to_create)
        .genome_maker(genome_maker)
        .scorer(scorer)
        .inspector(|solution_chunk| {
            update_best(&mut best, solution_chunk);
        })
        .parallel_search(true)
        .build();

    random_search.search()?;

    Ok(())
}

// We clearly don't want to copy these in lots of files – where should we put them?
fn update_best<Ge, Sc>(best: &mut Option<(usize, Ge, Sc)>, solution_chunk: &[(usize, Ge, Sc)])
where
    Ge: Clone + Display,
    Sc: Clone + Display + PartialOrd,
{
    for (sample_number, genome, score) in solution_chunk {
        match best {
            None => {
                let new_best = (*sample_number, genome.clone(), score.clone());
                print_best(&new_best);
                *best = Some(new_best);
            }
            Some((_, _, best_score)) => {
                if score > best_score {
                    let new_best = (*sample_number, genome.clone(), score.clone());
                    print_best(&new_best);
                    *best = Some(new_best);
                }
            }
        }
    }
}

fn print_best<Ge, Sc>((sample_number, genome, score): &(usize, Ge, Sc))
where
    Ge: Display,
    Sc: Display,
{
    println!(
        "New best solution found:  {:25} with error {:25} at sample number {:25}",
        genome, score, sample_number
    );
}
