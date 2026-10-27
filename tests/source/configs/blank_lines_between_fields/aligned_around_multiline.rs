//@ rustfmt-blank_lines_between_fields: AroundMultiline
//@ rustfmt-struct_field_align_threshold: 20

struct Aligned {
    a: u8,
    bbbbbb: u16,

    cc: u32,
    /// A documented field.
    ddddddddd: u64,
    e: u8,
    #[serde(default)]
    ffff: u8,
    gg: u8,
}
