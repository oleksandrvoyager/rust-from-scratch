//! Zero-copy lexer for a small env/Makefile-style config format.
//!
//! See `FORMAT.md` at the crate root for the grammar.

use std::collections::HashMap;
use std::fmt;

/// Error returned by [`tokenize`] for malformed input.
#[derive(Debug)]
pub enum TokenizeError {
    /// A non-empty, non-comment line has no `=`.
    MissingEquals,
    /// A `${` was never closed with a matching `}`.
    UnterminatedRef,
}

impl fmt::Display for TokenizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenizeError::MissingEquals => {
                write!(f, "line has no '=' separating a name from a value")
            }
            TokenizeError::UnterminatedRef => {
                write!(f, "'${{' was never closed with a matching '}}'")
            }
        }
    }
}

impl std::error::Error for TokenizeError {}

/// Error returned when resolving a name's value fails.
#[derive(Debug)]
pub enum ResolveError {
    /// A `${NAME}` reference points at a name with no definition.
    UndefinedReference(String),
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolveError::UndefinedReference(name) => {
                write!(f, "reference to undefined name '{name}'")
            }
        }
    }
}

impl std::error::Error for ResolveError {}

/// One piece recognized while scanning the source text.
#[derive(Debug, PartialEq)]
pub enum Token<'src> {
    /// The name on the left of `=`.
    Name(&'src str),
    /// The `=` separating a name from its value.
    Equals,
    /// A literal piece of a value.
    Text(&'src str),
    /// A `${OTHER_NAME}` reference within a value; carries just `OTHER_NAME`.
    Ref(&'src str),
}

/// One piece of a value, as stored per-name in a symbol table (see
/// [`build_symbol_table`]). Unlike [`Token`], only `Text`/`Ref` are
/// possible here — `Name`/`Equals` have already served their purpose.
#[derive(Debug, PartialEq)]
pub enum Segment<'src> {
    /// A literal piece of a value.
    Text(&'src str),
    /// A `${OTHER_NAME}` reference within a value; carries just `OTHER_NAME`.
    Ref(&'src str),
}

/// Splits `source` into tokens. See `FORMAT.md` for the grammar.
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

/// Groups a flat token stream by name: `Name`/`Equals` are consumed to find
/// the boundaries, and each name's `Text`/`Ref` pieces are collected under
/// it as [`Segment`]s.
pub fn build_symbol_table(tokens: Vec<Token<'_>>) -> HashMap<&'_ str, Vec<Segment<'_>>> {
    let mut symbol_table: HashMap<&'_ str, Vec<Segment<'_>>> = HashMap::new();
    let mut name = "";
    for token in tokens {
        match token {
            Token::Name(n) => {
                name = n;
                symbol_table.insert(name, vec![]);
            }
            Token::Equals => (),
            Token::Text(t) => {
                symbol_table.entry(name).or_default().push(Segment::Text(t));
            }
            Token::Ref(r) => {
                symbol_table.entry(name).or_default().push(Segment::Ref(r));
            }
        }
    }

    symbol_table
}

/// Resolves `name`'s value: literal segments are copied as-is, `Ref`
/// segments are resolved recursively and their result substituted in.
fn resolve(table: &HashMap<&str, Vec<Segment>>, name: &str) -> Result<String, ResolveError> {
    if let Some(tokens) = table.get(name) {
        let mut result = String::new();
        for token in tokens {
            match token {
                Segment::Text(text) => result.push_str(text),
                Segment::Ref(ref_name) => {
                    result.push_str(&resolve(table, ref_name)?);
                }
            }
        }

        Ok(result)
    } else {
        Err(ResolveError::UndefinedReference(name.to_owned()))
    }
}

/// Resolves every name in `table` to its final value, substituting all
/// `${OTHER_NAME}` references (including chained/nested ones).
pub fn resolve_all<'a>(
    table: &HashMap<&'a str, Vec<Segment<'a>>>,
) -> Result<HashMap<&'a str, String>, ResolveError> {
    let mut result = HashMap::new();
    for key in table.keys() {
        result.insert(*key, resolve(table, key)?);
    }

    Ok(result)
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

    #[test]
    fn build_symbol_table_groups_by_name() {
        let tokens = tokenize("HOST = localhost\nURL = https://${HOST}:8080/api").unwrap();
        let table = build_symbol_table(tokens);

        assert_eq!(table.len(), 2);
        assert_eq!(table["HOST"], vec![Segment::Text("localhost")]);
        assert_eq!(
            table["URL"],
            vec![
                Segment::Text("https://"),
                Segment::Ref("HOST"),
                Segment::Text(":8080/api"),
            ]
        );
    }

    #[test]
    fn build_symbol_table_keeps_names_with_empty_values() {
        let tokens = tokenize("EMPTY = ").unwrap();
        let table = build_symbol_table(tokens);

        assert_eq!(table.get("EMPTY"), Some(&vec![]));
    }

    #[test]
    fn resolve_all_substitutes_a_reference() {
        let tokens = tokenize("HOST = localhost\nURL = https://${HOST}:8080/api").unwrap();
        let table = build_symbol_table(tokens);
        let resolved = resolve_all(&table).unwrap();

        assert_eq!(resolved["HOST"], "localhost");
        assert_eq!(resolved["URL"], "https://localhost:8080/api");
    }

    #[test]
    fn resolve_all_follows_a_chain_of_references() {
        let tokens = tokenize("A = base\nB = ${A}-mid\nC = ${B}-top").unwrap();
        let table = build_symbol_table(tokens);
        let resolved = resolve_all(&table).unwrap();

        assert_eq!(resolved["C"], "base-mid-top");
    }

    #[test]
    fn resolve_all_reports_an_undefined_reference() {
        let tokens = tokenize("URL = ${MISSING}").unwrap();
        let table = build_symbol_table(tokens);

        assert!(matches!(
            resolve_all(&table),
            Err(ResolveError::UndefinedReference(name)) if name == "MISSING"
        ));
    }
}
