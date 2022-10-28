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
use rand::{Rng, rngs::ThreadRng};

use crate::io::{
    prompt,
    write,
    WriteArgs,
};

pub mod io;

const INVALID_CHOICE: &str = "Invalid choice!";
const MIN_SECRET: u8 = 0;
const MAX_SECRET: u8 = 100;

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
 * Menu
 *
 * Take an array of strings, then print them as choices in a menu. Finally, prompt user to choose
 * one of the choices by entering a number, then return the value wrapped in a result.
 */
fn menu(
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

/*
 * gen_num
 *
 * generate a secret number between MIN_SECRET & MAX_SECRET
 */
struct NumberGenerator {
    thread_rng: Option<ThreadRng>
}

impl NumberGenerator {
    fn new() -> Self {
        NumberGenerator {
            thread_rng: None,
        }
    }

    fn gen_secret(&mut self) -> u8 {
        self._get_rng().gen_range(MIN_SECRET, MAX_SECRET)
    }

    fn _get_rng(&mut self) -> ThreadRng {
        match self.thread_rng {
            Some(instance) => instance,
            None => self._init_rng(),
        }
    }

    fn _init_rng(&mut self) -> ThreadRng {
        self.thread_rng = Some(rand::thread_rng());
        self._get_rng()
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
    use std::{io::{self, Read }, fmt};

    use super::*;

    struct TestWriter {
        written_lines: Vec<String>,
        line_to_write: Option<String>,
    }

    impl TestWriter {
        fn new() -> TestWriter {
            TestWriter {
                written_lines: Vec::new(),
                line_to_write: None,
            }
        }

        fn append_to_line(&mut self, value: &str) {
            if let Some(line) = &self.line_to_write {
                let mut new = line.to_string();
                new.push_str(value);
                self.line_to_write = Some(new)
            } else {
                self.line_to_write = Some(value.to_string());
            }
        }
    }

    impl Write for TestWriter {
        fn write(&mut self, _s: &[u8]) -> io::Result<usize> {
            unimplemented!()
        }

        fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> io::Result<()> {
            struct Adapter<'a> {
                inner: &'a mut TestWriter,
                error: io::Result<()>,
            }

            impl fmt::Write for Adapter<'_> {
                fn write_str(&mut self, s: &str) -> fmt::Result {
                    self.inner.append_to_line(s);
                    Ok(())
                }
            }

            let mut output = Adapter { inner: self, error: Ok(()) };
            match fmt::write(&mut output, fmt) {
                Ok(()) => Ok(()),
                Err(..) => {
                    if output.error.is_err() {
                        output.error
                    } else {
                        Err(io::Error::new(io::ErrorKind::Other, "formatter error"))
                    }
                }
            }
        }

        fn flush(&mut self)  -> Result<(), io::Error> {
            if let Some(line) = &self.line_to_write {
                self.written_lines.push(line.as_str().to_string());
                self.line_to_write = None;
                Ok(())
            } else {
                Err(io::Error::new(io::ErrorKind::Other, "Nothing to write!"))
            }
        }
    }

    #[derive(Debug)]
    enum ReaderValues {
        One(String),
        Many(Vec<String>),
    }

    struct TestReader {
        values: ReaderValues,
        next_call: usize,
    }

    impl TestReader {
        fn new(values: ReaderValues) -> TestReader {
            TestReader {
                values,
                next_call: 0,
            }
        }
    }

    impl BufRead for TestReader {
        fn consume(&mut self, _amt: usize) {
            unimplemented!()
        }

        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            unimplemented!()
        }

        fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
            match &self.values {
                ReaderValues::One(value) => {
                    buf.push_str(value.as_str());
                    Ok(buf.len())
                },
                ReaderValues::Many(values) => {
                    if let Some(value) = values.get(self.next_call) {
                        self.next_call += 1;
                        buf.push_str(value.as_str());
                        Ok(buf.len())
                    } else {
                        Err(io::Error::new(io::ErrorKind::Other, "No more values to read."))
                    }
                },
            }
        }
    }

    impl Read for TestReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            unimplemented!()
        }

        fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
            match &self.values {
                ReaderValues::One(value) => {
                    buf.push_str(value.as_str());
                    Ok(buf.len())
                },
                ReaderValues::Many(values) => {
                    if let Some(value) = values.get(self.next_call) {
                        self.next_call += 1;
                        buf.push_str(value.as_str());
                        Ok(buf.len())
                    } else {
                        Err(io::Error::new(io::ErrorKind::Other, "No more values to read."))
                    }
                },
            }
        }
    }

    fn setup_io() -> (TestWriter, TestReader) {
        setup_io_with_input("1")
    }

    fn setup_io_with_input(input: &str) -> (TestWriter, TestReader) {
        let writer = TestWriter::new();
        let reader = TestReader::new(ReaderValues::One( String::from(input) ));

        (writer, reader)
    }

    fn setup_io_with_many_inputs(inputs: &[&str]) -> (TestWriter, TestReader) {
        let writer = TestWriter::new();

        let values: Vec<String> = inputs.iter()
            .map(|input| String::from(*input))
            .collect();
        let reader = TestReader::new(ReaderValues::Many(values));

        (writer, reader)
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
