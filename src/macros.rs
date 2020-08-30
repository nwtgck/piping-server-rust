// (from: https://stackoverflow.com/a/34324856/2885946)
#[macro_export]
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

#[macro_export]
macro_rules! with_values {
    (impl $name:ident {
        $(pub const $field_name:ident: $field_type:ty = $value:expr;)*
    }) => {
        impl $name {
            $(pub const $field_name: $field_type = $value;)*
            pub const VALUES: [&'static str; count!($($value)*)] = [$($value),*];
        }
    }
}
