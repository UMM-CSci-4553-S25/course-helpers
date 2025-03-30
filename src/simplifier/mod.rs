pub mod drop_one;

use rand::Rng;

pub trait Simplifier<G> {
    fn simplify_genome<R: Rng>(&self, genome: G, rng: &mut R) -> G;
}
