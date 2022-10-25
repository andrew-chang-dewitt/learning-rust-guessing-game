use std::{
    io::{
        self,
        BufRead,
        Write,
    },
    fmt::Arguments,
    cmp::Ordering, num::ParseIntError
};
use rand::Rng;

const INVALID_CHOICE: &str = "Invalid choice!";

/*
 * Main
 *
 * Gets I/O streams & sets up loop for running game
 */
fn main() {
    // get stdin & stdout reader & writer
    let mut output = io::stdout();
    let stdin = io::stdin();
    let mut input = stdin.lock();
    // get random number generator
    let mut rnd = rand::thread_rng();

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
                        let game_result = play_game(&mut rnd, &mut output, &mut input);
                        if let Err(value) = game_result {
                            match value {
                                GameError::Quit => {
                                    write(&mut output, WriteArgs::Str("You quit."));
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
 * Prompt
 *
 * get user input from stdin & return it as a String
 */
fn prompt(mut writer: impl Write, mut reader: impl BufRead) -> String {
    let mut answer = String::new();

    // print the prompt char
    write(&mut writer, WriteArgs::Str( "> " ));

    // get the user's response
    reader.read_line(&mut answer).unwrap();

    // pad w/ empty line
    write(&mut writer, WriteArgs::Str( "\n" ));

    answer.trim().to_string()
}

enum WriteArgs<'a> {
    Fmt(Arguments<'a>),
    Str(&'a str),
}

/*
 * Write
 *
 * Wrapper on using something that implements Write to output to a stream.
 * Takes either a &str or a set of formatted args as the output value.
 */
fn write(mut writer: impl Write, args: WriteArgs) {
    match args {
        WriteArgs::Fmt(x) =>
            writer.write_fmt(x).unwrap(),
        WriteArgs::Str(x) =>
            writer.write_fmt(format_args!("{}", x)).unwrap(),
    }
    writer.flush().unwrap();
}

/*
 * gen_num
 *
 * generage a random number between 1 & 100
 */
fn gen_num(rnd: &mut impl Rng) -> u8 {
    return rnd.gen_range(0,100);
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
enum GameError {
    Quit,
    Unknown,
}

fn play_game(
    mut rnd: impl Rng,
    mut writer: impl Write,
    mut reader: impl BufRead,
) -> Result<(), GameError> {
    // generate secret number
    let secret = gen_num(&mut rnd);

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
                keep_guessing = false;
                write(&mut writer, WriteArgs::Str("Quitting..."));

                if let "quit" = guess_value.as_str() {
                    res = Err(GameError::Quit);
                } else {
                    res = Err(GameError::Unknown);
                }
            }
        }
    }

    res
}

#[cfg(test)]
mod tests {
    use std::{io::Read, fmt};

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

    struct TestReader {
        value: String
    }

    impl TestReader {
        fn new(value: String) -> TestReader {
            TestReader {
                value
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
            buf.push_str(&self.value);
            Ok(buf.len())
        }
    }

    impl Read for TestReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            unimplemented!()
        }

        fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
            buf.push_str(&self.value);
            Ok(buf.len())
        }
    }

    fn setup() -> (TestWriter, TestReader) {
        setup_with_input("1")
    }

    fn setup_with_input(input: &str) -> (TestWriter, TestReader) {
        let writer = TestWriter::new();
        let reader = TestReader::new(String::from(input));

        (writer, reader)
    }

    #[test]
    fn menu_prints_generic_first_line() {
        let ( mut writer, reader ) = setup();
        let choices = ["first", "second"];
        menu(&choices, &mut writer, reader).unwrap();

        assert!(writer.written_lines.get(0).unwrap().contains("Please choose from the following..."));
    }

    #[test]
    fn menu_passes_given_choices_to_given_print_fn() {
        let ( mut writer, reader ) = setup();
        let choices = ["first", "second"];
        menu(&choices, &mut writer, reader).unwrap();

        assert!(writer.written_lines.get(1).unwrap().contains("first"));
        assert!(writer.written_lines.get(2).unwrap().contains("second"));
    }

    #[test]
    fn menu_returns_user_input() {
        let ( writer, reader ) = setup_with_input("1");
        let choices = ["choice"];
        let response = menu(&choices, writer, reader);

        assert_eq!(response.unwrap(), 1)
    }

    #[test]
    #[should_panic( expected = "Invalid choice!" )]
    fn menu_returns_error_if_user_input_is_not_a_number() {
        let ( writer, reader ) = setup_with_input("not a number");
        let choices = ["choice"];
        menu(&choices, writer, reader).unwrap();
    }

    #[test]
    #[should_panic( expected = "Invalid choice!" )]
    fn menu_returns_error_if_user_input_is_negative() {
        let ( writer, reader ) = setup_with_input("-1");
        let choices = ["choice"];
        menu(&choices, writer, reader).unwrap();
    }

    #[test]
    #[should_panic( expected = "Invalid choice!" )]
    fn menu_returns_error_if_user_input_is_higher_than_length_of_choice_array() {
        let ( writer, reader ) = setup_with_input("2");
        let choices = ["choice"];
        menu(&choices, writer, reader).unwrap();
    }

    #[test]
    #[should_panic( expected = "Invalid choice!" )]
    fn menu_returns_error_if_user_input_is_0() {
        let ( writer, reader ) = setup_with_input("0");
        let choices = ["choice"];
        menu(&choices, writer, reader).unwrap();
    }

    #[test]
    fn prompt_sends_prompt_char_to_given_print_fn() {
        let ( mut writer, reader ) = setup();
        prompt(&mut writer, reader);

        assert_eq!(writer.written_lines.get(0), Some( &( "> ").to_string() ));
    }

    #[test]
    fn prompt_returns_user_input() {
        let ( writer, reader ) = setup_with_input("given input");
        let actual = prompt(writer, reader);

        assert_eq!(actual, String::from("given input"))
    }

    #[test]
    fn gen_num_returns_number_from_0_to_100() {
        for _ in 0..10^100 {
            let actual = gen_num(&mut rand::thread_rng());
            let max = 100;
            assert!(actual <= max, "{actual} is not less than {max}")
        }
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
}
