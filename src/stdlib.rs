#[macro_export]
macro_rules! ink {
    ($($arg:tt)*) => {
        print!($($arg)*);
    };
}
