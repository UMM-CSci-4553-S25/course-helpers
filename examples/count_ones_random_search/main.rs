use std::thread;

use course_helpers::{
    channel_consumer::ChannelConsumer,
    processor::{
        min_max_distance::MinMaxDistance, print_best_solutions::PrintBestSolution, Processor,
    },
    random_search::{RandomSearch, RandomSearchError},
};
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
    let num_bits = 64;

    let channel_capacity = 1_000;
    let (sender, receiver) = flume::bounded(channel_capacity);

    let num_to_create = 1_000_000;
    let scorer = FnScorer(|bitstring: &Bitstring| count_ones(&bitstring.bits));

    let monitor = PrintBestSolution::default();
    let summarizer = MinMaxDistance::default();

    let mut all_monitors = (monitor, summarizer);

    let monitor_handle = thread::spawn(move || {
        all_monitors.consume_all(receiver);
        all_monitors.finalize_and_print();
    });

    // Create a `Distribution` that generates `Bitstring`s when sampled
    let genome_maker = StandardUniform.into_collection_generator(num_bits);

    let mut random_search = RandomSearch::new(num_to_create, genome_maker, scorer, sender.clone());
    random_search.run_to_end()?;

    drop(sender);

    monitor_handle.join().unwrap();

    Ok(())
}
