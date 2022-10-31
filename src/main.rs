use std::io::{stdin, stdout};

use crate::{
    constants::INVALID_CHOICE,
    game::{Game, GameError},
    io::{write, WriteArgs},
    menu::menu,
    random::NumberGenerator,
};

pub mod constants;
pub mod game;
pub mod io;
pub mod menu;
pub mod random;

/// Main
///
/// Get I/O streams & set up loop for running game repeatedly
fn main() {
    // get stdin & stdout reader & writer
    let mut output = stdout();
    let stdin = stdin();
    let mut input = stdin.lock();
    // get secret number generator
    let mut rnd = NumberGenerator::new();

    // greet the user
    write(
        &mut output,
        WriteArgs::Str("Welcome to the guessing game!\n\n"),
    );

    // set up the loop
    let mut playing: bool = true;
    // enter loop
    while playing {
        // render menu
        let choices = ["play game", "exit"];
        let res = menu(&choices, &mut output, &mut input);

        // init new game
        let secret = rnd.gen_secret();
        let mut game = Game::new(secret, &mut output, &mut input);

        // handle user choice
        match res {
            Ok(choice) => {
                match choice {
                    // play game -> enter game
                    1 => {
                        let game_result = game.play();
                        if let Err(value) = game_result {
                            match value {
                                GameError::Quit => {
                                    write(&mut output, WriteArgs::Str("You quit. "));
                                }
                                GameError::Unknown => {
                                    write(
                                        &mut output,
                                        WriteArgs::Str("An unknown Error occurred."),
                                    );
                                }
                            }
                        } else {
                            write(&mut output, WriteArgs::Str("You won!\n"));
                        }

                        write(&mut output, WriteArgs::Str("Play again?\n"));
                    }
                    // exit -> exit loop
                    2 => playing = false,
                    // not an allowable input
                    _ => println!("{}", INVALID_CHOICE),
                }
            }
            Err(reason) => println!("{}", reason),
        }
    }
}
