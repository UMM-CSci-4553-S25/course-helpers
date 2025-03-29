#![expect(
    clippy::arithmetic_side_effects,
    reason = "The tradeoff safety <> ease of writing arguably lies on the ease of writing side \
              for example code."
)]

pub mod args;

use clap::Parser;
use ec_core::{
    distributions::collection::ConvertToCollectionGenerator,
    generation::Generation,
    individual::{ec::WithScorer, scorer::FnScorer},
    operator::{
        genome_extractor::GenomeExtractor,
        genome_scorer::GenomeScorer,
        mutator::Mutate,
        selector::{best::Best, lexicase::Lexicase, tournament::Tournament, Select, Selector},
        Composable,
    },
    test_results::{self, TestResults},
    uniform_distribution_of,
};
use ec_linear::mutator::umad::Umad;
use miette::ensure;
use num_traits::Float;
use ordered_float::OrderedFloat;
use push::{
    evaluation::{Case, Cases, WithTargetFn},
    genome::plushy::{ConvertToGeneGenerator, Plushy},
    instruction::{variable_name::VariableName, FloatInstruction, PushInstruction},
    push_vm::{program::PushProgram, push_state::PushState, HasStack, State},
};
use rand::{prelude::Distribution, rng};

use crate::args::{CliArgs, RunModel};

/*
 * This is an implementation of the symbolic regression problem used as an example
 * in DEAP:
 * https://github.com/DEAP/deap/blob/master/examples/gp/symbreg.py
 */

// The penalty value to use when an evolved program doesn't have an expected
// "return" value on the appropriate stack at the end of its execution.
const PENALTY_VALUE: f64 = 1_000.0;

// Just so we don't have to type "OrderedFloat<f64>" over and over…
type Of64 = OrderedFloat<f64>;

/// The target polynomial is x^4 + x^3 + x^2 + x
fn target_fn(input: Of64) -> Of64 {
    input.powi(4) + input.powi(3) + input.powi(2) + input
}

// This is used to build the initial state of the Push interpreter.
// It will have the given program on the exec stack and have the
// input variable `x` associated with the given `input` value.
fn build_push_state(
    program: impl DoubleEndedIterator<Item = PushProgram> + ExactSizeIterator,
    input: Of64,
) -> PushState {
    #[expect(
        clippy::unwrap_used,
        reason = "This will panic if the program is longer than the allowed max stack size. We \
                  arguably should check that and return an error here."
    )]
    PushState::builder()
        .with_max_stack_size(1000)
        .with_program(program)
        .unwrap()
        .with_float_input("x", input)
        .build()
}

// Score the given `program` on the given `input`/`output` pair.
fn score_program(
    program: impl DoubleEndedIterator<Item = PushProgram> + ExactSizeIterator,
    Case { input, output }: Case<Of64>,
) -> Of64 {
    let state = build_push_state(program, input);

    let Ok(state) = state.run_to_completion() else {
        // Do some logging, perhaps?
        return Of64::from(PENALTY_VALUE);
    };

    let Ok(&answer) = state.stack::<Of64>().top() else {
        // Do some logging, perhaps?
        return Of64::from(PENALTY_VALUE);
    };

    // Square the error
    (answer - output).powi(2)
}

fn score_genome(
    genome: &Plushy,
    training_cases: &Cases<Of64>,
) -> TestResults<test_results::Error<Of64>> {
    let program: Vec<PushProgram> = genome.clone().into();

    training_cases
        .iter()
        .map(|&case| score_program(program.iter().cloned(), case))
        .collect()
}

fn main() -> miette::Result<()> {
    // FIXME: Respect the max_genome_length input
    let CliArgs {
        run_model,
        population_size,
        max_initial_instructions,
        max_generations,
        ..
    } = CliArgs::parse();

    let mut rng = rng();

    // Inputs from -1 (inclusive) to 1 (exclusive) in increments of 0.1.
    let training_cases = (-10..10)
        .map(|n| Of64::from(n) / 10.0)
        .with_target_fn(|&i| target_fn(i));

    /*
     * The `scorer` will need to take an evolved program (sequence of
     * instructions) and run it on all the inputs from -4 (inclusive) to 4
     * (exclusive) in increments of 0.25, collecting together the errors,
     * i.e., the absolute difference between the returned value and the
     * expected value.
     */
    let scorer = FnScorer(|genome: &Plushy| score_genome(genome, &training_cases));

    // Switching from tournament selection to lexicase selection will allow for better high success
    // rates even if you start with very small initial programs (e.g., length 1), which can increase
    // readability of the results even without simplification.

    // let selector = Lexicase::new(training_cases.len());
    // This uses a higher tournament size (10) than that used in DEAP (which uses 3). Populations
    // of Push programs tend to have a higher diversity of behaviors than populations of trees,
    // so here we apparently need a higher tournament size to ensure we're selecting and maintaining
    // the best individuals. I think that adding elitism would accomplish much the same thing.
    let selector = Tournament::of_size::<10>();

    let gene_generator = uniform_distribution_of![<PushInstruction>
        FloatInstruction::Add,
        FloatInstruction::Subtract,
        FloatInstruction::Multiply,
        FloatInstruction::ProtectedDivide,
        // FloatInstruction::Dup,
        // FloatInstruction::Push(OrderedFloat(0.0)),
        // FloatInstruction::Push(OrderedFloat(1.0)),
        VariableName::from("x")
    ]
    .into_gene_generator();

    let population = gene_generator
        .to_collection_generator(max_initial_instructions)
        .with_scorer(scorer)
        .into_collection_generator(population_size)
        .sample(&mut rng);

    ensure!(
        !population.is_empty(),
        "An initial populaiton is always required"
    );

    let best = Best.select(&population, &mut rng)?;
    println!("Best initial individual is {best}");

    // Use the UMAD (Uniform Mutation through Addition and Deletion) mutation operator.
    // We don't have any crossover operator set up for Push at the moment, so we'll just
    // use mutation.
    let umad = Umad::new(0.1, 0.1, &gene_generator);

    let make_new_individual = Select::new(selector)
        .then(GenomeExtractor)
        .then(Mutate::new(umad))
        .wrap::<GenomeScorer<_, _>>(scorer);

    let mut generation = Generation::new(make_new_individual, population);

    // TODO: It might be useful to insert some kind of logging system so we can
    // make this less imperative in nature.

    for generation_number in 0..max_generations {
        match run_model {
            RunModel::Serial => generation.serial_next()?,
            RunModel::Parallel => generation.par_next()?,
        }

        let best = Best.select(generation.population(), &mut rng)?;
        // TODO: Change 2 to be the smallest number of digits needed for
        // max_generations-1.
        println!("Generation {generation_number:2} best is {best}");

        if best.test_results.total_result.0 == OrderedFloat(0.0) {
            println!("SUCCESS");
            break;
        }
    }

    Ok(())
}
