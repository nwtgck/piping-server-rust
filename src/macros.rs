// (from: https://stackoverflow.com/a/34324856/2885946)
#[macro_export]
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + $crate::count!($($xs)*));
}

#[macro_export]
macro_rules! head {
    ($x:tt $($y:tt)*) => {
        $x
    };
}

#[macro_export]
macro_rules! with_values {
    (
        $(pub const $field_name:ident: $field_type:ty = $value:expr;)*
    ) => {
        $(pub const $field_name: $field_type = $value;)*
        pub const VALUES: [$crate::head!($($field_type)*); $crate::count!($($value)*)] = [$($value),*];
    }
}
