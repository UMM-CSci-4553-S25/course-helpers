/// Consume all the values on a given channel, processing
/// them as appropriate.
///
/// The values on the channels are presumably items generated
/// by a search process, probably either individual solutions or
/// collections of solutions (i.e., populations). The consumer
/// will process these values in some way, such as printing
/// them to the console, computing statistics, or storing them in a database.
/// Implementations of the [`Processor`](crate::processor::Processor) trait are typically how
/// the values are processed, so you probably want to implement that
/// trait to provide the processing logic.
///
/// In most cases you can just use the default
/// implementation of `ChannelConsumer` for the [`Processor`](crate::processor::Processor) trait,
/// so you're not likely to need to implement this trait explicitly. See
/// [the `Processor` documentation](crate::processor::Processor#the-default-implementation-of-channelconsumer-for-processor)
/// for an example of how this works.
///
pub trait ChannelConsumer<S> {
    /// Consume all the values on the provided channel, processing
    /// them as appropriate.
    fn consume_all(&mut self, receiver: flume::Receiver<S>);
}
