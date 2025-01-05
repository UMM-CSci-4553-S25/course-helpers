use crate::processor::Processor;

pub struct Consumer {}

impl Consumer {
    pub fn consume<S>(receiver: flume::Receiver<S>, processor: &mut impl Processor<S>) {
        while let Ok(solution) = receiver.recv() {
            processor.process(&solution);
        }
    }
}
