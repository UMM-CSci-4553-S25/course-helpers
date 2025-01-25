use std::fmt::Display;

use course_helpers::random_search::{RandomSearch, RandomSearchError};
use ec_core::{individual::scorer::FnScorer, test_results::Error};

fn main() -> Result<(), RandomSearchError> {
    let num_to_create = 1_000_000;
    let target = 589;
    let scorer = FnScorer(|value: &i64| Error(value.abs_diff(target)));

    // Create a `Distribution` that generates `i64`s when sampled
    let genome_maker = rand::distr::StandardUniform;

    let mut best = None;

    // let mut random_search = RandomSearch::new(num_to_create, genome_maker, scorer, sender.clone());
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

    let (best_sample_number, best_genome, best_score) = best.unwrap();
    println!(
        "Best solution found: sample_number: {}, genome: {:?}, score: {}",
        best_sample_number, best_genome, best_score
    );

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
