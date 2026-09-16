//! Lazy iterator over web server access log lines (Common Log Format).
//!
//! See `FORMAT.md` at the crate root for the grammar.

pub struct LogEntry<'a> {
    // TODO: which fields, and what types?
}

#[derive(Debug)]
pub enum ParseError {
    // TODO: what can go wrong parsing one line?
}

pub struct LogLines<'a> {
    // TODO: what does this need to remember between one `.next()` call and the next?
}

impl<'a> Iterator for LogLines<'a> {
    type Item = Result<LogEntry<'a>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
