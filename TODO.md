TODO
===

- [x] refactor how random number generation is passed around so it's easier to mock

  instead of passing something that impls `Rng` to a `gen_num` fn & calling
  `gen_range` directly, let's pass a `get_secret_number` fn around and mock
  that guy instead.

- [ ] write tests for `play_game`
  - [x] test first guess correct
  - [x] correct after some n guesses
  - [ ] quit
  - [ ] invalid input as guess

- [ ] refactor `play_game` fn to `Game` struct
