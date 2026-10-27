//@ rustfmt-blank_lines_between_variants: Never

enum Simple {
    A,

    B,
    C,
    /// A documented variant.
    D,
    E,
    // A comment on its own line.
    F,
    #[cfg(test)]
    G,
    H, // A trailing comment.
    I,
}

enum WithFields {
    A,
    B { x: u8, y: u8 },


    C(u8),
    D {
        a_long_field_name: SomeLongTypeName,
        another_long_field_name: AnotherLongTypeName,
    },
    E,
}

enum Discriminants {
    A = 1,

    B = 2,
    C = 3,
}
