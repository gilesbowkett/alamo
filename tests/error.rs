// error_chain: flatten an error and its source() chain into one message.
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct Wrap {
    msg: &'static str,
    source: Option<Box<dyn Error>>,
}

impl fmt::Display for Wrap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.msg)
    }
}

impl Error for Wrap {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref()
    }
}

#[test]
fn formats_error_and_its_source_chain() {
    let inner = Wrap { msg: "connection reset", source: None };
    let middle = Wrap { msg: "error decoding response body", source: Some(Box::new(inner)) };
    let outer = Wrap { msg: "GET failed", source: Some(Box::new(middle)) };

    assert_eq!(
        alamo::error_chain(&outer),
        "GET failed: error decoding response body: connection reset"
    );
}

#[test]
fn single_error_with_no_source_is_just_its_message() {
    let e = Wrap { msg: "boom", source: None };
    assert_eq!(alamo::error_chain(&e), "boom");
}
