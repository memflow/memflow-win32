/// Generates a regex string from an ida-like hex string.
/// Main benefit of this is that you can directly use your ida signatures,
/// while not needing to decode them at runtime or manually beforehand.
#[macro_export]
macro_rules! ida_regex {
    ( $($token:tt)+ ) => {
        concat!(
            "(?-u)", // Search with unicode code-points disabled. \\x4C is actually equal to the hex byte 0x4C
            $(
                $crate::ida_regex_part!($token)
            ),*
        )
    };
}

#[macro_export]
macro_rules! ida_regex_part {
    (?) => {
        "(?s:.)"
    };
    ($hex:tt) => {
        concat!("\\x", stringify!($hex))
    };
}
