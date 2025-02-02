use course_helpers::{inspector::update_best, random_search::RandomSearch};
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

fn main() {
    let num_to_create = 1_000_000;

    let num_bits = 32;

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

    random_search.search();
}
