#![expect(
    clippy::arithmetic_side_effects,
    reason = "The tradeoff safety <> ease of writing arguably lies on the ease of writing side \
              for example code."
)]

pub mod args;

use clap::Parser;
use course_helpers::simplifier::{drop_one::DropOne, Simplifier};
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
use push::{
    evaluation::{Case, Cases, WithTargetFn},
    genome::plushy::{ConvertToGeneGenerator, Plushy},
    instruction::{
        printing::{Print, PrintString},
        variable_name::VariableName,
        BoolInstruction, ExecInstruction, IntInstruction, PushInstruction,
    },
    push_vm::{program::PushProgram, push_state::PushState, State},
};
use rand::{prelude::Distribution, rng, Rng};
use strsim::damerau_levenshtein;

use crate::args::{CliArgs, RunModel};

/*
 * This is an implementation of the "Fizz Buzz" benchmark problem from
 * "PSB2 - the second program synthesis benchmark suite" by Thomas Helmuth and Peter Kelly
 * https://dl.acm.org/doi/10.1145/3449639.3459285
 *
 * Fizz Buzz (CW) Given an integer 𝑥, return (sic) "Fizz" if 𝑥 is divisible by 3,
 * "Buzz" if 𝑥 is divisible by 5, "FizzBuzz" if 𝑥 is divisible by 3 and 5,
 * and a string version of 𝑥 if none of the above hold.
 *
 * Because (as of April 2025) we don't have a `String` stack, I've implemented
 * this using printing instead of a return.
 */

// The penalty value to use when an evolved program doesn't have an expected
// "return" value on the appropriate stack at the end of its execution.
const PENALTY_VALUE: i128 = 1_000_000;

/// The target polynomial is (x^3 + 1)^3 + 1
/// i.e., x^9 + 3x^6 + 3x^3 + 2
fn target_fn(x: i64) -> String {
    if x % 3 == 0 && x % 5 == 0 {
        return "FizzBuzz".to_string();
    }
    if x % 3 == 0 {
        return "Fizz".to_string();
    }
    if x % 5 == 0 {
        return "Buzz".to_string();
    }
    x.to_string()
}

fn build_push_state(
    program: impl DoubleEndedIterator<Item = PushProgram> + ExactSizeIterator,
    x: i64,
) -> PushState {
    #[expect(
        clippy::unwrap_used,
        reason = "This will panic if the program is longer than the allowed max stack size. We \
                  arguably should check that and return an error here."
    )]
    PushState::builder()
        .with_max_stack_size(1000)
        .with_instruction_step_limit(1000)
        .with_program(program)
        .unwrap()
        .with_int_input("x", x)
        .build()
}

fn score_program(
    program: impl DoubleEndedIterator<Item = PushProgram> + ExactSizeIterator,
    Case { input, output }: &Case<i64, String>,
) -> i128 {
    let state = build_push_state(program, *input);

    let Ok(mut state) = state.run_to_completion() else {
        // Do some logging, perhaps?
        return PENALTY_VALUE;
    };

    let answer = state.stdout_string().unwrap();
    let expected = output.to_string();

    damerau_levenshtein(&answer, &expected) as i128
}

fn score_genome(
    genome: &Plushy,
    training_cases: &Cases<i64, String>,
) -> TestResults<test_results::Error<i128>> {
    let program: Vec<PushProgram> = genome.clone().into();

    training_cases
        .iter()
        .map(|case| score_program(program.iter().cloned(), case))
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

    // 40 random pairs, with the integer in the range -100..100 and the float between 0 and 1.
    let training_cases = std::iter::repeat_with(|| rng.random_range(0..1_000_000))
        .take(100)
        .chain(0..10)
        .chain((1..10).map(|x| x * 3))
        .chain((1..10).map(|x| x * 5))
        .chain((1..10).map(|x| x * 15))
        .with_target_fn(|&x| target_fn(x));

    println!("{training_cases:?}");

    /*
     * The `scorer` will need to take an evolved program (sequence of
     * instructions) and run it on all the inputs from -4 (inclusive) to 4
     * (exclusive) in increments of 0.25, collecting together the errors,
     * i.e., the absolute difference between the returned value and the
     * expected value.
     */
    let scorer = FnScorer(|genome: &Plushy| score_genome(genome, &training_cases));

    // If we use Lexicase selection instead of tournament selection, individual
    // generations will be slower, but we are much more likely to find a
    // solution to this problem.
    let selector = Lexicase::new(training_cases.len());

    // Using tournament selection on this problem leads to significantly lower
    // likelihood of success than if we use lexicase selection.
    // let selector = Tournament::of_size::<10>();

    let gene_generator = uniform_distribution_of![<PushInstruction>
        IntInstruction::Add,
        IntInstruction::Subtract,
        IntInstruction::Multiply,
        IntInstruction::ProtectedDivide,
        IntInstruction::Mod,
        IntInstruction::IsZero,
        IntInstruction::Print(Print::new()),
        IntInstruction::push(3),
        IntInstruction::push(5),
        VariableName::from("x"),

        BoolInstruction::And,
        BoolInstruction::Or,
        BoolInstruction::push(true),
        BoolInstruction::push(false),

        PushInstruction::PrintString(PrintString("Fizz".to_string())),
        PushInstruction::PrintString(PrintString("Buzz".to_string())),
        // Comment this out to see if we can evolve solutions without the combined string
        // constant being provided.
        PushInstruction::PrintString(PrintString("FizzBuzz".to_string())),

        ExecInstruction::if_else(),
    ]
    .into_gene_generator();

    let population = gene_generator
        .to_collection_generator(max_initial_instructions)
        .with_scorer(scorer)
        .into_collection_generator(population_size)
        .sample(&mut rng);

    ensure!(
        !population.is_empty(),
        "An initial population is always required"
    );

    let mut best = Best.select(&population, &mut rng)?.clone();
    println!("Best initial individual is {best}");

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

        best = Best.select(generation.population(), &mut rng)?.clone();
        // TODO: Change 2 to be the smallest number of digits needed for
        // max_generations-1.
        println!("Generation {generation_number:2} best is {best}");

        if best.test_results.total_result.0 == 0 {
            println!("SUCCESS");
            break;
        }
    }

    // TODO: This should also be removed (or the number of simplifications set to 0) when
    // doing timing comparisons since DEAP doesn't do anything like simplification.

    let drop_one_simplifier = DropOne::new(scorer, 10_000, 0);
    let simplified_best = drop_one_simplifier.simplify_genome(best.genome.clone(), &mut rng);
    println!("Simplified best is {simplified_best}");

    println!("The best results vector: {:?}", best.test_results.results);

    Ok(())
}
