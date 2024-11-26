pub mod prelude {
    use crate::PoolBuilder;

    pub trait Testing
    where
        Self: Sized,
    {
        fn reset_db(self, b: bool) -> Self;
    }

    impl Testing for PoolBuilder {
        fn reset_db(mut self, b: bool) -> Self {
            self.reset_db = b;
            self
        }
    }
}
