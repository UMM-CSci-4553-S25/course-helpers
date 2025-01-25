use std::fmt::Display;

// We clearly don't want to copy these in lots of files – where should we put them?
pub fn update_best<Ge, Sc>(best: &mut Option<(usize, Ge, Sc)>, solution_chunk: &[(usize, Ge, Sc)])
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

pub fn print_best<Ge, Sc>((sample_number, genome, score): &(usize, Ge, Sc))
where
    Ge: Display,
    Sc: Display,
{
    println!(
        "New best solution found:  {:25} with error {:25} at sample number {:25}",
        genome, score, sample_number
    );
}
