use std::thread;

use course_helpers::{
    channel_consumer::ChannelConsumer,
    processor::{min_max_distance::MinMaxDistance, print_monitor::PrintMonitor, Processor},
    random_search::{RandomSearch, RandomSearchError},
};
use ec_core::individual::scorer::FnScorer;

fn main() -> Result<(), RandomSearchError> {
    let channel_capacity = 1_000;
    let (sender, receiver) = flume::bounded(channel_capacity);

    let num_to_create = 10_000;
    // let scorer = DistanceFromTarget::new(589);
    let target = 589;
    let scorer = FnScorer(|value: &i64| value.abs_diff(target));

    let monitor = PrintMonitor::default();
    let summarizer = MinMaxDistance::default();

    let mut all_monitors = (monitor, summarizer);

    let monitor_handle = thread::spawn(move || {
        all_monitors.consume_all(receiver);
        all_monitors.finalize_and_print();
    });

    // Create a `Distribution` that generates `i64`s when sampled
    let genome_maker = rand::distr::StandardUniform;

    let mut random_search = RandomSearch::new(num_to_create, genome_maker, scorer, sender.clone());
    random_search.run_to_end()?;

    drop(sender);

    monitor_handle.join().unwrap();

    Ok(())
}
