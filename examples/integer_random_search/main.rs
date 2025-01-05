use std::thread;

use course_helpers::{
    consumer::Consumer,
    processor::{min_max_distance::MinMaxDistance, print_monitor::PrintMonitor, Processor},
    random_search::{RandomSearch, RandomSearchError},
    scorer::distance_from_target::DistanceFromTarget,
};

fn main() -> Result<(), RandomSearchError> {
    let channel_capacity = 1_000;
    let (sender, receiver) = flume::bounded(channel_capacity);

    let num_to_create = 10_000;
    let scorer = DistanceFromTarget::new(589);

    let monitor = PrintMonitor::default();
    let summarizer = MinMaxDistance::default();

    let mut all_monitors = (monitor, summarizer);

    let monitor_handle = thread::spawn(move || {
        Consumer::consume(receiver, &mut all_monitors);
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