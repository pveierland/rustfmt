//@ rustfmt-blank_lines_between_fields: Never

struct Simple {
    a: u8,
    b: u8,
    c: u8,
    /// A documented field.
    d: u8,
    e: u8,
    // A comment on its own line.
    f: u8,
    #[serde(default)]
    g: u8,
    h: u8, // A trailing comment.
    i: u8,
}

struct Wrapped {
    a: u8,
    a_field_with_a_long_name:
        SomeVeryLongTypeName<WithGenericParameters, AndMore, AndEvenMore, AndStillMoreParameters>,
    b: u8,
}

union Union {
    a: u8,
    b: u16,
}

enum Enum {
    Variant {
        a: u8,
        /// A documented field.
        b: u8,
        c: u8,
    },
}

struct OneLine {
    a: u8,
    b: u8,
}

fn literal() {
    let s = Simple {
        a: first_value_with_a_long_name,

        b: 2,
        c: third_value_with_a_long_name,
    };
}
