use std::io::{
    Write,
    BufRead
};

use crate::constants::*;
use crate::io::{
    prompt,
    write,
    WriteArgs,
};
#[cfg(test)]
use crate::io::test_utils::{
    setup_io,
    setup_io_with_input,
};

/// Take an array of strings and print them as choices in a menu. Then,
/// prompt user to choose one of the choices by entering a number. Finally,
/// return the value wrapped in a result.
pub fn menu(
    choices: &[&str],
    mut writer: impl Write,
    mut reader: impl BufRead
) -> Result<usize, &'static str> {
    write(&mut writer, WriteArgs::Str( "\nPlease choose from the following...\n" ));

    for (index, choice) in choices.iter().enumerate() {
        write(
            &mut writer,
            WriteArgs::Fmt( format_args!( "{}) {}\n", index + 1, choice ))
        );
    }

    let choice: Result<usize, _> = prompt(&mut writer, &mut reader).parse();

    if let Ok(num) = choice {
        if num > 0 && num <= choices.len() {
            Ok(num)
        } else {
            Err(INVALID_CHOICE)
        }
    } else { Err(INVALID_CHOICE) }
}

#[test]
fn menu_prints_generic_first_line() {
    let ( mut writer, reader ) = setup_io();
    let choices = ["first", "second"];
    menu(&choices, &mut writer, reader).unwrap();

    assert!(writer.written_lines.get(0).unwrap().contains("Please choose from the following..."));
}

#[test]
fn menu_passes_given_choices_to_given_print_fn() {
    let ( mut writer, reader ) = setup_io();
    let choices = ["first", "second"];
    menu(&choices, &mut writer, reader).unwrap();

    assert!(writer.written_lines.get(1).unwrap().contains("first"));
    assert!(writer.written_lines.get(2).unwrap().contains("second"));
}

#[test]
fn menu_returns_user_input() {
    let ( writer, reader ) = setup_io_with_input("1");
    let choices = ["choice"];
    let response = menu(&choices, writer, reader);

    assert_eq!(response.unwrap(), 1)
}

#[test]
#[should_panic( expected = "Invalid choice!" )]
fn menu_returns_error_if_user_input_is_not_a_number() {
    let ( writer, reader ) = setup_io_with_input("not a number");
    let choices = ["choice"];
    menu(&choices, writer, reader).unwrap();
}

#[test]
#[should_panic( expected = "Invalid choice!" )]
fn menu_returns_error_if_user_input_is_negative() {
    let ( writer, reader ) = setup_io_with_input("-1");
    let choices = ["choice"];
    menu(&choices, writer, reader).unwrap();
}

#[test]
#[should_panic( expected = "Invalid choice!" )]
fn menu_returns_error_if_user_input_is_higher_than_length_of_choice_array() {
    let ( writer, reader ) = setup_io_with_input("2");
    let choices = ["choice"];
    menu(&choices, writer, reader).unwrap();
}

#[test]
#[should_panic( expected = "Invalid choice!" )]
fn menu_returns_error_if_user_input_is_0() {
    let ( writer, reader ) = setup_io_with_input("0");
    let choices = ["choice"];
    menu(&choices, writer, reader).unwrap();
}
