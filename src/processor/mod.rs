pub mod min_max_distance;
pub mod print_monitor;

pub trait Processor<S> {
    fn process(&mut self, solution: &S);

    fn finalize_and_print(&self) {}
}

impl<S, A, B> Processor<S> for (A, B)
where
    A: Processor<S>,
    B: Processor<S>,
{
    fn process(&mut self, solution: &S) {
        self.0.process(solution);
        self.1.process(solution);
    }

    fn finalize_and_print(&self) {
        self.0.finalize_and_print();
        self.1.finalize_and_print();
    }
}