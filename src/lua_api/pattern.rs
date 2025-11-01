use std::fmt;
use std::num::ParseIntError;

use crate::lua_api::input_types as it;

#[derive(Debug)]
pub enum ParseError {
    Empty,
    InvalidStatePrefix(char),
    InvalidTimeout(ParseIntError),
    InvalidToken(String),
    UpNotAllowedForChord(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Empty => write!(f, "empty pattern"),
            ParseError::InvalidStatePrefix(c) => write!(f, "invalid state prefix: {}", c),
            ParseError::InvalidTimeout(e) => write!(f, "invalid timeout value: {}", e),
            ParseError::InvalidToken(s) => write!(f, "invalid token: {}", s),
            ParseError::UpNotAllowedForChord(s) => write!(f, "'^' (Up) not allowed for chord: {}", s),
        }
    }
}

impl std::error::Error for ParseError {}

/// Token は pattern の最小単位（Hotkey または Chord）を表現する。
/// ここでは input_types::{Hotkey, State, KeySpec 相当} を再利用する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Hotkey(it::Hotkey),
    Chord { state: it::State, keys: Vec<it::Hotkey> },
}

/// 文字列の pattern をパースして Vec<Token> を返す
/// pattern はスペース区切りでトークン（シーケンス）を分ける
pub fn parse_pattern(s: &str) -> Result<Vec<Token>, ParseError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(ParseError::Empty);
    }

    let mut tokens: Vec<Token> = Vec::new();
    s.split(',').map(|raw_token| {
        raw_token.split('-').map(|raw_key| {

        })
    });

    Ok(tokens)
}
