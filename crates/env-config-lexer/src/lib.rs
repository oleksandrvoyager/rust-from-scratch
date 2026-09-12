//! Zero-copy lexer for a small env/Makefile-style config format.
//!
//! See `FORMAT.md` at the crate root for the grammar.

#[derive(Debug)]
pub enum TokenizeError {
    MissingEquals,
    UnterminatedRef,
}

#[derive(Debug, PartialEq)]
pub enum Token<'src> {
    Name(&'src str),
    Equals,
    Text(&'src str),
    Ref(&'src str),
}

pub fn tokenize(source: &str) -> Result<Vec<Token<'_>>, TokenizeError> {
    let mut tokens = Vec::new();
    for line in str::lines(source) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(e) = line.find("=") {
            tokens.push(Token::Name(line[..e].trim()));
            tokens.push(Token::Equals);

            let mut remaining = line[e + 1..].trim();
            loop {
                if let Some(x) = remaining.find("${") {
                    let (text, rest) = remaining.split_at(x);
                    if !text.trim().is_empty() {
                        tokens.push(Token::Text(text.trim()));
                    }

                    if let Some(e) = rest.find("}") {
                        let (token_ref, rest) = rest.split_at(e + 1);
                        tokens.push(Token::Ref(token_ref[2..e].trim()));

                        remaining = rest.trim();
                    } else {
                        return Err(TokenizeError::UnterminatedRef);
                    }
                } else {
                    if !remaining.is_empty() {
                        tokens.push(Token::Text(remaining));
                    }
                    break;
                }
            }
        } else {
            return Err(TokenizeError::MissingEquals);
        }
    }

    Ok(tokens)
}
