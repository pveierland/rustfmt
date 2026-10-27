//@ rustfmt-blank_lines_between_match_arms: Never

fn simple(x: u8) {
    match x {
        0 => a(),
        1 => b(),
        2 => {
            c();
            d()
        }
        3 => e(),
        // A comment on its own line.
        4 => f(),
        #[cfg(test)]
        5 => g(),
        6 => h(), // A trailing comment.
        7 => i(),
        _ => j(),
    }
}

fn nested(x: Option<u8>) -> u8 {
    match x {
        Some(y) => match y {
            0 => 1,
            _ => 2,
        },
        None => 0,
    }
}

fn guards(x: u8) -> u8 {
    match x {
        y if y > 10
            && some_long_condition_that_does_not_fit(y)
            && another_condition_here(y)
            && yet_another(y) =>
        {
            1
        }
        _ => 0,
    }
}
