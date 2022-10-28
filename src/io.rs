use std::{
    fmt::Arguments, 
    io::{
        Write,
        BufRead
    }
};

/*
 * Prompt
 *
 * get user input from stdin & return it as a String
 */
pub fn prompt(mut writer: impl Write, mut reader: impl BufRead) -> String {
    let mut answer = String::new();

    // print the prompt char
    write(&mut writer, WriteArgs::Str( "> " ));

    // get the user's response
    reader.read_line(&mut answer).unwrap();

    // pad w/ empty line
    write(&mut writer, WriteArgs::Str( "\n" ));

    answer.trim().to_string()
}

pub enum WriteArgs<'a> {
    Fmt(Arguments<'a>),
    Str(&'a str),
}

/*
 * Write
 *
 * Wrapper on using something that implements Write to output to a stream.
 * Takes either a &str or a set of formatted args as the output value.
 */
pub fn write(mut writer: impl Write, args: WriteArgs) {
    match args {
        WriteArgs::Fmt(x) =>
            writer.write_fmt(x).unwrap(),
        WriteArgs::Str(x) =>
            writer.write_fmt(format_args!("{}", x)).unwrap(),
    }
    writer.flush().unwrap();
}
