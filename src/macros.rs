#[macro_export]
macro_rules! suite {
    ($seed:expr) => {
        $crate::suite::Suite::new($seed)
    };
    () => {
        suite!(0)
    };
}
