use std::{
    fmt::Arguments,
    io::{BufRead, Write},
};

/// Get user input from write stream (e.g. stdin) & return it as a String.
///
/// Takes a write stream & a read stream, implementing Write & BufRead.
///
/// # Example
///
/// // get stdin & stdout reader & writer
/// let mut output = stdout();
/// let stdin = stdin();
/// let mut input = stdin.lock();
/// let user_input = prompt(&mut output, &mut input)
/// // do something with input...
pub fn prompt(mut writer: impl Write, mut reader: impl BufRead) -> String {
    let mut answer = String::new();

    // print the prompt char
    write(&mut writer, WriteArgs::Str("> "));

    // get the user's response
    reader.read_line(&mut answer).unwrap();

    // pad w/ empty line
    write(&mut writer, WriteArgs::Str("\n"));

    answer.trim().to_string()
}

/// The types of data allowable as output to give to `write()`
///
/// Either a set of Format Arguments or a string slice.
///
/// # Example
///
/// let args = WriteArgs::Fmt(format_args!("{}", "Hello World"));
/// let slice = WriteArgs::Str("A slice");
pub enum WriteArgs<'a> {
    Fmt(Arguments<'a>),
    Str(&'a str),
}

/// Writes given args to given write stream.
///
/// Used to encapsulate writing with dependency injection to make it more easily
/// testable.
///
/// # Example
/// // get stdin & stdout reader & writer
/// let mut output = stdout();
/// write(&mut output, WriteArgs::Str("Hello World!"))
/// // prints "Hello World!" to stdout
pub fn write(mut writer: impl Write, args: WriteArgs) {
    match args {
        WriteArgs::Fmt(x) => writer.write_fmt(x).unwrap(),
        WriteArgs::Str(x) => writer.write_fmt(format_args!("{}", x)).unwrap(),
    }
    writer.flush().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_sends_prompt_char_to_given_print_fn() {
        let (mut writer, reader) = test_utils::setup_io();
        prompt(&mut writer, reader);

        assert_eq!(writer.written_lines.get(0), Some(&("> ").to_string()));
    }

    #[test]
    fn prompt_returns_user_input() {
        let (writer, reader) = test_utils::setup_io_with_input("given input");
        let actual = prompt(writer, reader);

        assert_eq!(actual, String::from("given input"))
    }
}

/// Testing utilities for working with Write & Read streams
#[cfg(test)]
pub mod test_utils {
    use std::{
        fmt::{write, Arguments, Result as FmtResult, Write as FmtWrite},
        io::{BufRead, Error, ErrorKind, Read, Result as IoResult, Write},
    };

    /// Setup a Write stream & a Read stream for testing with user input of "1"
    ///
    /// # Example
    ///
    /// let ( mut writer, reader ) = setup_io();
    ///
    /// write(&mut writer, WriteArgs::Str("Hello World!"));
    /// assert_eq!(
    ///     writer.written_lines.get(0).unwrap(),
    ///     String::from("Hello World!")
    /// );
    ///
    /// let user_input = prompt(&mut writer, reader);
    /// assert_eq!(
    ///     user_input,
    ///     String::from("1")
    /// );
    pub fn setup_io() -> (TestWriter, TestReader) {
        setup_io_with_input("1")
    }

    /// Setup a Write stream & a Read stream for testing with given string as the
    /// simulated user input
    ///
    /// # Example
    ///
    /// let ( mut writer, reader ) = setup_io_with_input("This is input");
    ///
    /// let user_input = prompt(&mut writer, reader);
    /// assert_eq!(
    ///     user_input,
    ///     String::from("This is input")
    /// );
    pub fn setup_io_with_input(input: &str) -> (TestWriter, TestReader) {
        let writer = TestWriter::new();
        let reader = TestReader::new(ReaderValues::One(String::from(input)));

        (writer, reader)
    }

    /// Setup a Write stream & a Read stream for testing with given list of
    /// strings to be read as simulated user input in index order.
    ///
    /// # Example
    ///
    /// let inputs = ["first input", "second input", "last input"];
    /// let ( mut writer, reader ) = setup_io_with_many_inputs(&inputs);
    ///
    /// assert_eq!(
    ///     prompt(&mut writer, reader),
    ///     Ok(String::from("first input"))
    /// );
    /// assert_eq!(
    ///     prompt(&mut writer, reader),
    ///     Ok(String::from("second input"))
    /// );
    /// assert_eq!(
    ///     prompt(&mut writer, reader),
    ///     Ok(String::from("last input"))
    /// );
    pub fn setup_io_with_many_inputs(inputs: &[&str]) -> (TestWriter, TestReader) {
        let writer = TestWriter::new();

        let values: Vec<String> = inputs.iter().map(|input| String::from(*input)).collect();
        let reader = TestReader::new(ReaderValues::Many(values));

        (writer, reader)
    }

    pub struct TestWriter {
        pub written_lines: Vec<String>,
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
        fn write(&mut self, _s: &[u8]) -> IoResult<usize> {
            unimplemented!()
        }

        fn write_fmt(&mut self, fmt: Arguments<'_>) -> IoResult<()> {
            struct Adapter<'a> {
                inner: &'a mut TestWriter,
                error: IoResult<()>,
            }

            impl FmtWrite for Adapter<'_> {
                fn write_str(&mut self, s: &str) -> FmtResult {
                    self.inner.append_to_line(s);
                    Ok(())
                }
            }

            let mut output = Adapter {
                inner: self,
                error: Ok(()),
            };
            match write(&mut output, fmt) {
                Ok(()) => Ok(()),
                Err(..) => {
                    if output.error.is_err() {
                        output.error
                    } else {
                        Err(Error::new(ErrorKind::Other, "formatter error"))
                    }
                }
            }
        }

        fn flush(&mut self) -> Result<(), Error> {
            if let Some(line) = &self.line_to_write {
                self.written_lines.push(line.as_str().to_string());
                self.line_to_write = None;
                Ok(())
            } else {
                Err(Error::new(ErrorKind::Other, "Nothing to write!"))
            }
        }
    }

    #[derive(Debug)]
    enum ReaderValues {
        One(String),
        Many(Vec<String>),
    }

    pub struct TestReader {
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

        fn fill_buf(&mut self) -> IoResult<&[u8]> {
            unimplemented!()
        }

        fn read_line(&mut self, buf: &mut String) -> IoResult<usize> {
            match &self.values {
                ReaderValues::One(value) => {
                    buf.push_str(value.as_str());
                    Ok(buf.len())
                }
                ReaderValues::Many(values) => {
                    if let Some(value) = values.get(self.next_call) {
                        self.next_call += 1;
                        buf.push_str(value.as_str());
                        Ok(buf.len())
                    } else {
                        Err(Error::new(ErrorKind::Other, "No more values to read."))
                    }
                }
            }
        }
    }

    impl Read for TestReader {
        fn read(&mut self, _buf: &mut [u8]) -> IoResult<usize> {
            unimplemented!()
        }

        fn read_to_string(&mut self, buf: &mut String) -> IoResult<usize> {
            match &self.values {
                ReaderValues::One(value) => {
                    buf.push_str(value.as_str());
                    Ok(buf.len())
                }
                ReaderValues::Many(values) => {
                    if let Some(value) = values.get(self.next_call) {
                        self.next_call += 1;
                        buf.push_str(value.as_str());
                        Ok(buf.len())
                    } else {
                        Err(Error::new(ErrorKind::Other, "No more values to read."))
                    }
                }
            }
        }
    }
}
