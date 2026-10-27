//@ rustfmt-blank_lines_between_statements: Never

fn single_line_statements() {
    let a = 1;
    let b = 2;
    foo(a, b);
}

fn multiline_statements() -> Result<Vec<Item>, Error> {
    let first = 1;
    let items = source
        .iter()
        .filter(|item| item.is_valid() && item.is_enabled())
        .map(|item| item.transform())
        .collect::<Vec<_>>();
    let second = 2;
    if first > second {
        return Err(Error::new());
    }
    for item in &items {
        process(item);
    }
    let third = 3;
    Ok(items)
}

fn existing_blank_lines() {
    let a = 1;
    if a > 0 {
        foo();
    }
    bar();
}

fn comments() {
    let a = 1; // A trailing comment.
    if a > 0 {
        foo();
    }
    // A comment attached to the next statement.
    if a > 1 {
        bar();
    }
    /* A block
     * comment. */
    let b = 2;
    match b {
        _ => {}
    }
}

fn nested() {
    let closure = || {
        let a = 1;
        if a > 0 {
            foo();
        }
        a
    };
    loop {
        let b = 2;
        while b > 0 {
            bar();
        }
        baz();
    }
}

fn leading_items() {
    use std::collections::HashMap;
    use std::fmt::{self, Debug, Display, Formatter, Result as FmtResult, Write as FmtWrite};
    struct Local {
        a: u8,
    }
    let local = Local { a: 1 };
}

fn skipped() {
    let a = 1;
    #[rustfmt::skip]
    let b = [
        1, 0,
        0, 1,
    ];
    let c = 3;
}

fn macros() {
    let a = 1;
    println!(
        "a value that is long enough to wrap the macro call {} {} {}",
        a, a, a
    );
    let b = 2;
}

fn commented_statements() {
    let config = load();
    // Retry once, as the first connection attempt may time out.
    let connection = connect(&config);
    run(connection);
    let a = 1; // A trailing comment does not belong to the next statement.
    let b = 2;
    // A comment which is separated from the next statement.
    let c = 3;
    let d = 4;
}

fn first_commented_statement() {
    // A comment at the start of a block.
    let a = 1;
    let b = 2;
}
