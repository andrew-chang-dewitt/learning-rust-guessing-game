use std::{
    cmp::Ordering,
    io::{
        BufRead,
        Write,
        stdout,
        stdin
    },
    num::ParseIntError,
};

use crate::constants::*;
use crate::io::{
    prompt,
    write,
    WriteArgs,
};
use crate::menu::menu;
use crate::random::NumberGenerator;

pub mod constants;
pub mod io;
pub mod menu;
pub mod random;

/*
 * Main
 *
 * Gets I/O streams & sets up loop for running game
 */
fn main() {
    // get stdin & stdout reader & writer
    let mut output = stdout();
    let stdin = stdin();
    let mut input = stdin.lock();
    // get secret number generator
    let mut rnd = NumberGenerator::new();

    // greet the user
    write(&mut output, WriteArgs::Str( "Welcome to the guessing game!\n\n" ));

    // set up the loop
    let mut playing: bool = true;
    // enter loop
    while playing {
        // render menu
        let choices = ["play game", "exit"];
        let res = menu(&choices, &mut output, &mut input);

        // handle user choice
        match res {
            Ok(choice) => {
                match choice {
                    // play game -> enter game
                    1 => {
                        let game_result = play_game(|| rnd.gen_secret(), &mut output, &mut input);
                        if let Err(value) = game_result {
                            match value {
                                GameError::Quit => {
                                    write(&mut output, WriteArgs::Str("You quit. "));
                                },
                                GameError::Unknown => {
                                    write(
                                        &mut output,
                                        WriteArgs::Str("An unknown Error occurred.")
                                    );
                                },
                            }
                        } else {
                            write(&mut output, WriteArgs::Str("You won!\n"));
                        }

                        write(&mut output, WriteArgs::Str("Play again?\n"));
                    },
                    // exit -> exit loop
                    2 => playing = false,
                    // not an allowable input
                    _ => println!("{}", INVALID_CHOICE),
                }
            },
            Err(reason) => println!("{}", reason),
        }
    }
}

/*
 * evaluate
 *
 * Compare two numbers and return Ok if equal, otherwise Err with value of too high or too low if
 * not equal.
 */
fn evaluate(actual: u8, expected: u8) -> Result<(), String> {
    match actual.cmp(&expected) {
        Ordering::Equal => Ok(()),
        Ordering::Less => Err(format!("{} is too low!", actual)),
        Ordering::Greater => Err(format!("{} is too high!", actual)),
    }
}

/*
 * play_game
 *
 * Main function for starting a game round. Gets a secret number, then starts a loop prompting the
 * Guesser to guess in each iteration. Continues looping until the Guesser submits a correct guess.
 * Returns Ok when the loop ends. Exits loop early & returns Err if user enters "quit" instead of
 * a guess.
 */
#[derive(Debug)]
enum GameError {
    Quit,
    Unknown,
}

fn play_game(
    mut get_secret: impl FnMut() -> u8,
    mut writer: impl Write,
    mut reader: impl BufRead,
) -> Result<(), GameError> {
    // generate secret number
    let secret = get_secret();
    // create variable to store game result
    let mut res: Result<(), GameError> = Err(GameError::Unknown);
    // set up loop
    let mut keep_guessing = true;

    while keep_guessing {
        // prompt for guess
        write(&mut writer, WriteArgs::Str("Guess a number...\n"));
        let guess_value = prompt(&mut writer, &mut reader);
        let guess_parsed: Result<u8, ParseIntError> =
            guess_value.parse();
        match guess_parsed {
            // if guess parses to int evaluate it
            Ok(guess) => {
                let evaluated = evaluate(guess, secret);

                if let Err(value) = evaluated {
                    write(&mut writer, WriteArgs::Fmt(format_args!("{}\n\n", value)))
                } else {
                    keep_guessing = false;
                    res = Ok(());
                    write(&mut writer, WriteArgs::Str("Correct! "));
                }
            },
            // return error if guess is "quit"
            Err(_) => {
                if let "quit" = guess_value.as_str() {
                    keep_guessing = false;
                    write(&mut writer, WriteArgs::Str("Quitting...\n"));
                    res = Err(GameError::Quit);
                } else {
                    write(
                        &mut writer,
                        WriteArgs::Str("Invalid input, please guess an integer belonging to [0,100] or enter 'quit' to quit playing.\n")
                    );
                }
            }
        }
    }

    res
}

#[cfg(test)]
mod tests {
    use crate::io::test_utils::{
        setup_io_with_input,
        setup_io_with_many_inputs,
    };

    use super::*;

    #[test]
    fn evaluate_returns_ok_if_guess_is_correct() -> Result<(), String> {
        evaluate(1, 1)
    }

    #[test]
    fn evaluate_returns_err_if_guess_is_incorrect() -> Result<(), ()> {
        match evaluate(1, 2) {
            Err(_) => Ok(()),
            _ => Err(()),
        }
    }

    #[test]
    fn evaluate_specifies_if_guess_is_too_high() {
        let reason = match evaluate(11, 10) {
            Err(reason) => reason,
            _ => panic!("evaluate should be Err"),
        };

        assert!(reason.contains("too high"))
    }

    #[test]
    fn evaluate_specifies_if_guess_is_too_low() {
        let reason = match evaluate(9, 10) {
            Err(reason) => reason,
            _ => panic!("evaluate should be Err"),
        };

        assert!(reason.contains("too low"))
    }

    #[test]
    fn play_game_returns_ok_if_guesser_is_correct_on_first_guess() {
        let ( writer, reader ) = setup_io_with_input("1");
        let test_secret = 1;
        let game_result = play_game(|| test_secret, writer, reader);

        match game_result {
            Ok(()) => assert!(true),
            Err(err) => assert!(false, "This shouldn't be Err {:?}", err),
        }
    }

    #[test]
    fn play_game_returns_ok_if_guesser_is_eventually_correct() {
        let guesses = ["0", "1"];
        let ( writer, reader ) = setup_io_with_many_inputs(&guesses);
        let test_secret = 1;
        let game_result = play_game(|| test_secret, writer, reader);

        match game_result {
            Ok(()) => assert!(true),
            Err(err) => assert!(false, "This shouldn't be Err {:?}", err),
        }
    }

    #[test]
    fn play_game_returns_quit_if_user_enters_quit() {
        let ( writer, reader ) = setup_io_with_input("quit");
        let test_secret = 1;
        let game_result = play_game(|| test_secret, writer, reader);

        match game_result {
            Ok(()) => assert!(false, "This should never happen"),
            Err(err) => {
                if let GameError::Quit = err {
                    assert!(true)
                } else {
                    assert!(false, "Err should contain 'quit', not '{:?}'", err)
                }
            },
        }
    }

    #[test]
    fn play_game_alerts_guesser_if_input_is_invalid() {
        let guesses = ["not a valid input", "1"];
        let ( mut writer, reader ) = setup_io_with_many_inputs(&guesses);
        let test_secret = 1;
        play_game(|| test_secret, &mut writer, reader);

        let invalid_input = writer.written_lines
            .iter()
            .find(|line| line.contains("Invalid input"));

        match invalid_input {
            Some(_) => assert!(true),
            None => assert!(false, "output should include line indicating first input was invalid"),
        }
    }

    #[test]
    fn play_game_allows_user_to_continue_guessing_after_invalid_input() {
        let guesses = ["not a valid input", "1"];
        let ( writer, reader ) = setup_io_with_many_inputs(&guesses);
        let test_secret = 1;
        let game_result = play_game(|| test_secret, writer, reader);

        match game_result {
            Ok(()) => assert!(true),
            Err(err) => assert!(false, "This shouldn't be Err {:?}", err),
        }
    }
}
