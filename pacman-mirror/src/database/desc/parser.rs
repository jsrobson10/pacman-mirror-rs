use std::{num::ParseIntError};

use thiserror::Error;

use crate::database::desc::cursor::Cursor;


#[derive(Debug,Error)]
pub enum ParseError {
    #[error("End of file: {cursor}")]
    Eof { cursor: Cursor },
    #[error("Unexpected character in name: {cursor}, expected: {expected:?}")]
    Name { cursor: Cursor, expected: char },
    #[error("Missing fields")]
    MissingFields,
    #[error("Parse {field}: {err}")]
    ParseInt { field: &'static str, err: ParseIntError },
    #[error("Decode {field}: {err}")]
    Decode { field: &'static str, err: hex::FromHexError },
}

pub fn parse(src: Box<str>, mut dst_func: impl FnMut(&str, &str)) -> Result<(), ParseError> {
    let mut it = src.char_indices();

    while let Some((idx, ch)) = it.next() {
        let name_start = idx + 1;
        let mut name = None;
        if ch != '%' {
            return Err(ParseError::Name { cursor: Cursor::new(src, idx), expected: '%' });
        }
        for (idx, ch) in it.by_ref() {
            match ch {
                '%' => {
                    name = Some(&src[name_start..idx]);
                    break;
                }
                '\n' => {
                    return Err(ParseError::Name { cursor: Cursor::new(src, idx), expected: '%' });
                }
                _ => {}
            }
        }
        let (Some(name), Some((idx, ch))) = (name, it.next()) else {
            let len = src.len();
            return Err(ParseError::Eof { cursor: Cursor::new(src, len) });
        };
        if ch != '\n' {
            return Err(ParseError::Name { cursor: Cursor::new(src, idx), expected: '\n' });
        }
        let body_start = idx + 1;
        let mut body = None;
        let mut found_nl = true;
        for (idx, ch) in it.by_ref() {
            match (ch, found_nl) {
                ('\n', true) => {
                    body = Some(&src[body_start..(idx-1)]);
                    break;
                }
                ('\n', false) => {
                    found_nl = true;
                }
                (_, true) => {
                    found_nl = false;
                }
                _ => {}
            }
        }
        let Some(body) = body else {
            continue;
        };
        dst_func(name, body);
    }
    Ok(())
}

