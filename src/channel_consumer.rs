pub trait ChannelConsumer<S> {
    fn consume_all(&mut self, receiver: flume::Receiver<S>);
}
