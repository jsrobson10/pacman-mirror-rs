use owning_ref::ArcRef;
use thiserror::Error;

use crate::database::desc::cursor::Cursor;


#[derive(Debug,Error)]
pub enum ParseError {
	#[error("End of file: {}", Cursor::new(src.clone(), src.len()))]
	Eof { src: ArcRef<str> },
	#[error("Unexpected character in name: {cursor}, expected: {expected:?}")]
	Name { cursor: Cursor, expected: char },
}

pub fn parse(src: ArcRef<str>, mut dst_func: impl FnMut(ArcRef<str>, ArcRef<str>)) -> Result<(), ParseError> {
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
					name = Some(src.clone().map(|v| &v[name_start..idx]));
					break;
				}
				'\n' => {
					return Err(ParseError::Name { cursor: Cursor::new(src, idx), expected: '%' });
				}
				_ => {}
			}
		}
		let (Some(name), Some((idx, ch))) = (name, it.next()) else {
			return Err(ParseError::Eof { src });
		};
		if ch != '\n' {
			return Err(ParseError::Name { cursor: Cursor::new(src, idx), expected: '\n' });
		}
		let body_start = idx + 1;
		let mut body = None;
		let mut found_nl = true;
		for (_, ch) in it.by_ref() {
			match (ch, found_nl) {
				('\n', true) => {
					body = Some(src.clone().map(|v| &v[body_start..]));
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

