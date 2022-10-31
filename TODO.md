TODO
===

- [x] refactor how random number generation is passed around so it's easier to mock

  instead of passing something that impls `Rng` to a `gen_num` fn & calling
  `gen_range` directly, let's pass a `get_secret_number` fn around and mock
  that guy instead.

- [x] write tests for `play_game`
  - [x] test first guess correct
  - [x] correct after some n guesses
  - [x] quit
  - [x] invalid input as guess

- [x] refactor game functionality to own module
  - [x] `play_game`
  - [x] `GameError`
  - [x] `evaluate`
  - [x] tests
- [x] refactor `play_game` & `evaluate` fns to `Game` struct
  - [x] takes a `secret`, `reader`, & `writer` on init
  - [x] exposes game play through public `play` method
  - [x] uses private `evaluate` method to evaluate guesses
- [ ] refactor `menu` fn to `Menu` struct
  - [ ] takes `reader` & `writer` on init
  - [ ] method for defining intro text
  - [ ] method for adding option text & handler
  - [ ] method for rendering menu
  - [ ] method for prompting user with menu & handling response
