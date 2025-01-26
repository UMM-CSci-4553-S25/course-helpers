use std::fmt::Display;

/// Updates `current_best` with any better solution found in `candidate_solutions`.
/// One solution is better than another if its score is higher.
///
/// # Examples
///
/// This initially sets `best` to be `None`, because we haven't yet seen
/// any potential solutions. After updating with two different chunks,
/// we should see `best` updated to new, improved values.
///
/// ```
/// # use course_helpers::inspector::update_best;
/// #
/// let mut best = None;
///
/// let first_chunk = [&(0, "a", 5), &(1, "b", 8), &(2, "c", 9)];
/// update_best(&mut best, &first_chunk);
/// assert_eq!(best, Some((2, "c", 9)));
///
/// let second_chunk = [&(3, "d", 2), &(4, "e", 11), &(5, "f", 4)];
/// update_best(&mut best, &second_chunk);
/// assert_eq!(best, Some((4, "e", 11)));
/// ```
pub fn update_best<Genome, Score>(
    current_best: &mut Option<(usize, Genome, Score)>,
    candidate_solutions: &[(usize, Genome, Score)],
) where
    Genome: Clone + Display,
    Score: Clone + Display + PartialOrd,
{
    for (sample_number, genome, score) in candidate_solutions {
        match current_best {
            None => {
                let new_best = (*sample_number, genome.clone(), score.clone());
                print_best(&new_best);
                *current_best = Some(new_best);
            }
            Some((_, _, best_score)) => {
                if score > best_score {
                    let new_best = (*sample_number, genome.clone(), score.clone());
                    print_best(&new_best);
                    *current_best = Some(new_best);
                }
            }
        }
    }
}

/// Print the given genome as the new best known solution.
pub fn print_best<Genome, Score>((sample_number, genome, score): &(usize, Genome, Score))
where
    Genome: Display,
    Score: Display,
{
    println!(
        "New best solution found:  {:25} with error {:25} at sample number {:25}",
        genome, score, sample_number
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_best_test() {
        let mut best = None;

        let first_chunk = [&(0, "a", 5), &(1, "b", 8), &(2, "c", 9)];
        update_best(&mut best, &first_chunk);
        assert_eq!(best, Some((2, "c", 9)));

        let second_chunk = [&(3, "d", 2), &(4, "e", 11), &(5, "f", 4)];
        update_best(&mut best, &second_chunk);
        assert_eq!(best, Some((4, "e", 11)));
    }
}
