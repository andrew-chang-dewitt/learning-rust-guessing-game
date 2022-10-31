use std::{
    cmp::Ordering,
    io::{BufRead, Write},
    num::ParseIntError,
};

use crate::io::{prompt, write, WriteArgs};

#[cfg(test)]
use crate::io::test_utils::{setup_io, setup_io_with_input, setup_io_with_many_inputs};

/// Types of Errors that can be returned at the end of a game. Quit is used to
/// indicate the user requested to quit the game, Unknown shouldn't happen,
/// but exists to cover unexpected behavior.
#[derive(Debug)]
pub enum GameError {
    Quit,
    Unknown,
}

/// Represents a game as an object that knows a secret number & exposes
/// a `play` method that prompts the guesser to guess in a loop until the
/// guess correctly.
pub struct Game<W: Write, R: BufRead> {
    reader: R,
    secret: u8,
    writer: W,
}

impl<W: Write, R: BufRead> Game<W, R> {
    /// Create a new Game instance with the given secret number & io streams.
    pub fn new(secret: u8, writer: W, reader: R) -> Self {
        Game {
            secret,
            writer,
            reader,
        }
    }

    /// Main function for starting a game round. Gets a secret number, then starts a
    /// loop prompting the Guesser to guess in each iteration. Continues looping
    /// until the Guesser submits a correct guess.  Returns Ok when the loop ends.
    /// Exits loop early & returns Err if user enters "quit" instead of a guess.
    pub fn play(&mut self) -> Result<(), GameError> {
        // create variable to store game result
        let mut res: Result<(), GameError> = Err(GameError::Unknown);
        // set up loop
        let mut keep_guessing = true;

        while keep_guessing {
            // prompt for guess
            write(&mut self.writer, WriteArgs::Str("Guess a number...\n"));
            let guess_value = prompt(&mut self.writer, &mut self.reader);
            let guess_parsed: Result<u8, ParseIntError> = guess_value.parse();
            match guess_parsed {
                // if guess parses to int evaluate it
                Ok(guess) => {
                    let evaluated = self.evaluate(guess);

                    if let Err(value) = evaluated {
                        write(
                            &mut self.writer,
                            WriteArgs::Fmt(format_args!("{}\n\n", value)),
                        )
                    } else {
                        keep_guessing = false;
                        res = Ok(());
                        write(&mut self.writer, WriteArgs::Str("Correct! "));
                    }
                }
                // return error if guess is "quit"
                Err(_) => {
                    if let "quit" = guess_value.as_str() {
                        keep_guessing = false;
                        write(&mut self.writer, WriteArgs::Str("Quitting...\n"));
                        res = Err(GameError::Quit);
                    } else {
                        write(
                            &mut self.writer,
                            WriteArgs::Str("Invalid input, please guess an integer belonging to [0,100] or enter 'quit' to quit playing.\n")
                        );
                    }
                }
            }
        }

        res
    }

    /// Compare two numbers and return Ok if equal, otherwise Err with value of too
    /// high or too low if not equal.
    fn evaluate(&self, actual: u8) -> Result<(), String> {
        match actual.cmp(&self.secret) {
            Ordering::Equal => Ok(()),
            Ordering::Less => Err(format!("{} is too low!", actual)),
            Ordering::Greater => Err(format!("{} is too high!", actual)),
        }
    }
}

#[cfg(test)]
mod test_utils {
    use crate::io::test_utils::{TestReader, TestWriter};

    use super::*;

    pub fn setup_game_with_secret(secret: u8) -> Game<TestWriter, TestReader> {
        let (writer, reader) = setup_io();
        Game::new(secret, writer, reader)
    }
}

#[test]
fn takes_secret_and_io_read_and_write_streams_on_init() -> Result<(), String> {
    let (mut writer, reader) = setup_io();
    let secret = 8;
    Game::new(secret, &mut writer, reader);
    Ok(())
}

#[test]
fn play_game_returns_ok_if_guesser_is_correct_on_first_guess() -> Result<(), GameError> {
    let (writer, reader) = setup_io_with_input("1");
    let test_secret = 1;
    let mut game = Game::new(test_secret, writer, reader);

    game.play()
}

#[test]
fn play_game_returns_ok_if_guesser_is_eventually_correct() -> Result<(), GameError> {
    let guesses = ["0", "1"];
    let (writer, reader) = setup_io_with_many_inputs(&guesses);
    let test_secret = 1;
    let mut game = Game::new(test_secret, writer, reader);

    game.play()
}

#[test]
fn play_game_returns_quit_if_user_enters_quit() -> Result<(), String> {
    let (writer, reader) = setup_io_with_input("quit");
    let test_secret = 1;
    let mut game = Game::new(test_secret, writer, reader);

    match game.play() {
        Ok(()) => Err(String::from("This should never happen")),
        Err(err) => {
            if let GameError::Quit = err {
                Ok(())
            } else {
                Err(format!("Err should contain 'quit', not '{:?}'", err))
            }
        }
    }
}

#[test]
fn play_game_alerts_guesser_if_input_is_invalid() -> Result<(), String> {
    let guesses = ["not a valid input", "1"];
    let (mut writer, reader) = setup_io_with_many_inputs(&guesses);
    let test_secret = 1;
    let mut game = Game::new(test_secret, &mut writer, reader);
    game.play()
        .map_err(|err| format!("Unexpected error: {:?}", err))?;

    let invalid_input = writer
        .written_lines
        .iter()
        .find(|line| line.contains("Invalid input"));

    match invalid_input {
        Some(_) => Ok(()),
        None => Err(String::from(
            "output should include line indicating first input was invalid",
        )),
    }
}

#[test]
fn play_game_allows_user_to_continue_guessing_after_invalid_input() -> Result<(), String> {
    let guesses = ["not a valid input", "1"];
    let (writer, reader) = setup_io_with_many_inputs(&guesses);
    let test_secret = 1;
    let mut game = Game::new(test_secret, writer, reader);

    game.play()
        .map_err(|err| format!("This shouldn't be Err {:?}", err))
}

#[test]
fn evaluate_returns_ok_if_guess_is_correct() -> Result<(), String> {
    let game = test_utils::setup_game_with_secret(1);
    game.evaluate(1)
}

#[test]
fn evaluate_returns_err_if_guess_is_incorrect() -> Result<(), String> {
    let game = test_utils::setup_game_with_secret(2);

    match game.evaluate(1) {
        Err(_) => Ok(()),
        _ => Err(String::from("This should have Errored")),
    }
}

#[test]
fn evaluate_specifies_if_guess_is_too_high() {
    let game = test_utils::setup_game_with_secret(10);
    let reason = match game.evaluate(11) {
        Err(reason) => reason,
        _ => panic!("evaluate should be Err"),
    };

    let expected = "too high";
    assert!(
        reason.contains(expected),
        "{reason} should contain {expected}"
    )
}

#[test]
fn evaluate_specifies_if_guess_is_too_low() {
    let game = test_utils::setup_game_with_secret(10);
    let reason = match game.evaluate(9) {
        Err(reason) => reason,
        _ => panic!("evaluate should be Err"),
    };

    let expected = "too low";
    assert!(
        reason.contains(expected),
        "{reason} should contain {expected}"
    )
}
