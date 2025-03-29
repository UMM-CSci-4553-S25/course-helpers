use clap::Parser;

#[derive(clap::ValueEnum, Copy, Clone, Debug)]
pub enum RunModel {
    Serial,
    Parallel,
}

/// Simple genetic algorithm in Rust
#[derive(Parser, Debug, Copy, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Should we use parallelism when doing the run?
    #[clap(short, long, value_enum, default_value_t = RunModel::Parallel)]
    pub run_model: RunModel,

    /// Population size
    #[clap(short, long, value_parser, default_value_t = 300)]
    pub population_size: usize,

    /// Maximum number of initial instructions
    #[clap(short = 'i', long, value_parser, default_value_t = 75)]
    pub max_initial_instructions: usize,

    /// Maximum genome length
    #[clap(short, long, value_parser, default_value_t = 1000)]
    pub max_genome_length: usize,

    /// Number of generations to run
    #[clap(short = 'g', long, value_parser, default_value_t = 40)]
    pub max_generations: usize,
}
