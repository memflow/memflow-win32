/// Generates a regex string from an ida-like hex string.
/// Made by [@ConnorBP] :)
/// Main benefit of this is that you can directly use your ida signatures,
/// while not needing to decode them at runtime or manually beforehand.
///
/// Example Usage:
/// ```
/// use regex::Regex;
/// use memflow_win32::ida_regex;
/// const REGEX_PATTERN: &str = ida_regex![4C 8B ? ? ? ? 3C 9F];
/// let re = Regex::new(REGEX_PATTERN).unwrap();
/// assert_eq!(REGEX_PATTERN, "(?-u)\\x4C\\x8B(?s:.){4}\\x3C\\x9F");
/// ```
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
