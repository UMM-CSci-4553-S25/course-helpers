use course_helpers::random_search::{RandomSearch, RandomSearchError};
use ec_core::individual::scorer::FnScorer;

fn print_best((sample_number, genome, score): (usize, i64, u64)) {
    println!(
        "New best solution found:  {:25} with error {:25} at sample number {:25}",
        genome, score, sample_number
    );
}

fn main() -> Result<(), RandomSearchError> {
    let num_to_create = 1_000_000;
    let target = 589;
    let scorer = FnScorer(|value: &i64| value.abs_diff(target));

    // Create a `Distribution` that generates `i64`s when sampled
    let genome_maker = rand::distr::StandardUniform;

    let mut best = None;

    // let mut random_search = RandomSearch::new(num_to_create, genome_maker, scorer, sender.clone());
    let mut random_search = RandomSearch::builder()
        .num_to_search(num_to_create)
        .genome_maker(genome_maker)
        .scorer(scorer)
        .inspector(|solution_chunk| {
            for &(sample_number, genome, score) in solution_chunk {
                match best {
                    None => {
                        best = Some((sample_number, genome.clone(), score));
                        print_best(best.unwrap());
                    }
                    Some((_, _, best_score)) => {
                        if score < best_score {
                            best = Some((sample_number, genome.clone(), score));
                            print_best(best.unwrap());
                        }
                    }
                }
            }
        })
        .parallel_search(false)
        .build();
    random_search.search()?;

    let (best_sample_number, best_genome, best_score) = best.unwrap();
    println!(
        "Best solution found: sample_number: {}, genome: {:?}, score: {}",
        best_sample_number, best_genome, best_score
    );

    Ok(())
}
