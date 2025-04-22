pub trait ServerRuntime {
    fn block_on<F>(&self, async_block: F)
    where
        F: Future<Output = ()> + Send + 'static;
    fn spawn<F>(future: F)
    where
        F: Future<Output = ()> + Send + 'static;
}