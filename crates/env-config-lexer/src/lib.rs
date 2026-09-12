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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_assignment() {
        let tokens = tokenize("HOST = localhost").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Name("HOST"), Token::Equals, Token::Text("localhost")]
        );
    }

    #[test]
    fn value_with_one_reference() {
        let tokens = tokenize("URL = https://${HOST}:8080/api").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Name("URL"),
                Token::Equals,
                Token::Text("https://"),
                Token::Ref("HOST"),
                Token::Text(":8080/api"),
            ]
        );
    }

    #[test]
    fn value_with_two_adjacent_references() {
        let tokens = tokenize("GREETING = ${A}-${B}").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Name("GREETING"),
                Token::Equals,
                Token::Ref("A"),
                Token::Text("-"),
                Token::Ref("B"),
            ]
        );
    }

    #[test]
    fn value_that_is_only_a_reference() {
        let tokens = tokenize("ALIAS = ${HOST}").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Name("ALIAS"), Token::Equals, Token::Ref("HOST"),]
        );
    }

    #[test]
    fn multiple_lines() {
        let tokens = tokenize("HOST = localhost\nPORT = 8080").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Name("HOST"),
                Token::Equals,
                Token::Text("localhost"),
                Token::Name("PORT"),
                Token::Equals,
                Token::Text("8080"),
            ]
        );
    }

    #[test]
    fn comments_and_blank_lines_are_ignored() {
        let tokens = tokenize("# a comment\n\nHOST = localhost\n  # indented comment\n").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Name("HOST"), Token::Equals, Token::Text("localhost")]
        );
    }

    #[test]
    fn line_without_equals_is_an_error() {
        assert!(matches!(
            tokenize("not_an_assignment"),
            Err(TokenizeError::MissingEquals)
        ));
    }

    #[test]
    fn unterminated_reference_is_an_error() {
        assert!(matches!(
            tokenize("URL = ${HOST"),
            Err(TokenizeError::UnterminatedRef)
        ));
    }
}
