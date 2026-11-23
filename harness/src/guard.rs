//! The plain-file guard. A submission that passes may be merged by program,
//! because it carries no prose for the next agent to read and no way to pull
//! in other files. Everything here works on tokens from rustc's own lexer.

use ra_ap_rustc_lexer::{tokenize, FrontmatterAllowed, LiteralKind, TokenKind};

pub const MAX_BYTES: usize = 256 * 1024;
pub const MAX_IDENT: usize = 32;

const MACROS: &[&str] = &["matches", "unreachable", "assert", "debug_assert"];
const ATTRIBUTES: &[&str] = &["inline", "repr", "derive", "allow", "cold", "must_use"];
const BANNED_WORDS: &[&str] = &["extern", "macro_rules", "macro"];
/// Keywords that may stand directly before a `!` that means "not".
const KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "else", "if", "in", "let", "match", "move", "mut", "ref",
    "return", "static", "unsafe", "where", "while",
];

struct Tok<'a> {
    kind: TokenKind,
    text: &'a str,
    line: usize,
}

/// Returns `Ok(())` when the file may be merged by program, or one line saying why not.
pub fn check(source: &[u8]) -> Result<(), String> {
    if source.is_empty() || source.len() > MAX_BYTES {
        return Err(format!(
            "guard: the file must be between 1 and {MAX_BYTES} bytes"
        ));
    }
    check_bytes(source)?;
    let text =
        std::str::from_utf8(source).map_err(|_| "guard: the file is not text".to_string())?;
    let tokens = lex(text);
    for (i, tok) in tokens.iter().enumerate() {
        let problem = token_rule(tok).or_else(|| sequence_rule(&tokens, i));
        if let Some(message) = problem {
            return Err(format!("guard: {message} (line {})", tok.line));
        }
    }
    Ok(())
}

fn check_bytes(source: &[u8]) -> Result<(), String> {
    let mut line = 1;
    for &b in source {
        if b == b'\n' {
            line += 1;
        } else if b != b'\t' && !(0x20..=0x7E).contains(&b) {
            return Err(format!(
                "guard: only printable ASCII, tabs and line feeds are allowed (line {line})"
            ));
        }
    }
    Ok(())
}

fn lex(text: &str) -> Vec<Tok<'_>> {
    let mut out = Vec::new();
    let (mut pos, mut line) = (0usize, 1usize);
    for token in tokenize(text, FrontmatterAllowed::No) {
        let end = pos + token.len as usize;
        let piece = &text[pos..end];
        if token.kind != TokenKind::Whitespace {
            out.push(Tok {
                kind: token.kind,
                text: piece,
                line,
            });
        }
        line += piece.bytes().filter(|&b| b == b'\n').count();
        pos = end;
    }
    out
}

/// Rules that look at one token alone.
fn token_rule(tok: &Tok<'_>) -> Option<String> {
    match tok.kind {
        TokenKind::LineComment { .. } | TokenKind::BlockComment { .. } => {
            Some("comments are not allowed".into())
        }
        TokenKind::Literal { kind, .. } => literal_rule(kind),
        TokenKind::Ident => ident_rule(tok.text),
        TokenKind::Lifetime { .. } if tok.text.len() > MAX_IDENT => Some(long_name()),
        TokenKind::Frontmatter { .. }
        | TokenKind::InvalidIdent
        | TokenKind::RawIdent
        | TokenKind::ForcedKeywordIdent
        | TokenKind::UnknownPrefix
        | TokenKind::UnknownPrefixLifetime
        | TokenKind::RawLifetime
        | TokenKind::GuardedStrPrefix
        | TokenKind::Dollar
        | TokenKind::Unknown => Some("this token is not allowed".into()),
        _ => None,
    }
}

fn literal_rule(kind: LiteralKind) -> Option<String> {
    match kind {
        LiteralKind::Int { .. } | LiteralKind::Float { .. } => None,
        LiteralKind::Char { terminated: true } | LiteralKind::Byte { terminated: true } => None,
        LiteralKind::Char { .. } | LiteralKind::Byte { .. } => {
            Some("this token is not allowed".into())
        }
        _ => Some("string literals are not allowed".into()),
    }
}

fn ident_rule(text: &str) -> Option<String> {
    if text.len() > MAX_IDENT {
        return Some(long_name());
    }
    BANNED_WORDS
        .iter()
        .find(|w| **w == text)
        .map(|w| format!("the word `{w}` is not allowed"))
}

fn long_name() -> String {
    format!("names are limited to {MAX_IDENT} characters")
}

/// Rules that need the neighbours of a token: macro calls and attributes.
fn sequence_rule(tokens: &[Tok<'_>], i: usize) -> Option<String> {
    let next = tokens.get(i + 1).map(|t| t.kind);
    match tokens[i].kind {
        TokenKind::Bang => macro_rule(tokens, i, next),
        TokenKind::Pound => attribute_rule(tokens, i, next),
        TokenKind::Ident if tokens[i].text == "as" => alias_rule(tokens.get(i + 1)),
        _ => None,
    }
}

/// `use core::include_bytes as assert;` would smuggle a banned macro in under an
/// allowed name. No allowed macro or attribute name may be the target of an `as`.
fn alias_rule(next: Option<&Tok<'_>>) -> Option<String> {
    let name = next.filter(|t| t.kind == TokenKind::Ident)?;
    if MACROS.contains(&name.text) || ATTRIBUTES.contains(&name.text) {
        return Some("a macro or attribute name cannot be used as an alias".into());
    }
    None
}

fn macro_rule(tokens: &[Tok<'_>], i: usize, next: Option<TokenKind>) -> Option<String> {
    let opens = matches!(
        next,
        Some(TokenKind::OpenParen | TokenKind::OpenBracket | TokenKind::OpenBrace)
    );
    let prev = i.checked_sub(1).map(|p| &tokens[p])?;
    if !opens || prev.kind != TokenKind::Ident || KEYWORDS.contains(&prev.text) {
        return None;
    }
    if MACROS.contains(&prev.text) {
        return None;
    }
    Some(format!(
        "only these macros are allowed: {}",
        MACROS.join(", ")
    ))
}

fn attribute_rule(tokens: &[Tok<'_>], i: usize, next: Option<TokenKind>) -> Option<String> {
    if next != Some(TokenKind::OpenBracket) {
        return Some("inner attributes are not allowed".into());
    }
    let name = tokens.get(i + 2)?;
    if name.kind == TokenKind::Ident && ATTRIBUTES.contains(&name.text) {
        return None;
    }
    Some(format!(
        "only these attributes are allowed: {}",
        ATTRIBUTES.join(", ")
    ))
}
