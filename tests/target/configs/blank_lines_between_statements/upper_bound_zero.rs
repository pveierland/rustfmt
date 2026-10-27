//@ rustfmt-blank_lines_between_statements: AroundMultiline
//@ rustfmt-blank_lines_upper_bound: 0
fn upper_bound_zero() {
    let a = 1;
    if a > 0 {
        foo();
    }
    bar();
}
