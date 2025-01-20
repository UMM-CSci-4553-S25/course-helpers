use ec_core::{
    individual::{ec::EcIndividual, Individual},
    population::Population,
};
use ec_linear::genome::bitstring::Bitstring;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

/// Compute the entropy of the given population of `EcIndividual<Bitstring, _>`.
///
/// To compute the entropy of the set of bitstrings:
///    - Compute the mean of each bit position
///    - Then sum up (mean * log_2(mean) + (1-mean)* log_2(1- mean)) across each position.
#[expect(
    clippy::cast_precision_loss,
    clippy::as_conversions,
    reason = "I'm happy just smashing the types for now."
)]
pub fn entropy<Score>(population: &[EcIndividual<Bitstring, Score>]) -> f64 {
    let pop_size = population.len();
    // To compute the entropy of the set of bitstrings:
    //    - Compute the mean of each bit position
    //    - Then sum up (mean * log_2(mean) + (1-mean)* log_2(1- mean)) across each position.
    let bitstrings = population
        .iter()
        .map(EcIndividual::genome)
        .collect::<Vec<_>>();
    let means = (0..bitstrings[0].bits.size()).into_par_iter().map(|index| {
        bitstrings
            .iter()
            .filter(|bitstring| bitstring.bits[index])
            .count() as f64
            / pop_size as f64
    });
    #[expect(
        clippy::suboptimal_flops,
        reason = "I'm not sure using `mul_add` buys us anything and makes it more confusing"
    )]
    means
        .map(|mean| {
            mean * (mean + f64::MIN_POSITIVE).log2()
                + (1.0 - mean) * (1.0 - mean + f64::MIN_POSITIVE).log2()
        })
        .sum()
}
