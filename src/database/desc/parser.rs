use std::io::BufRead;


pub struct DescParser<R> where R: BufRead {
	src: R,
}

impl<R> DescParser<R> where R: BufRead {
	pub fn new(src: R) -> Self {
		Self { src }
	}
}

impl<R> Iterator for DescParser<R> where R: BufRead {
	type Item = anyhow::Result<(String, String)>;
	fn next(&mut self) -> Option<Self::Item> {
		let mut field = String::new();
		let mut body = String::new();
		loop {
			match self.src.read_line(&mut field) {
				Ok(len) => match len {
					0 => return None,
					1 => continue,
					_ => break,
				}
				Err(err) => {
					return Some(Err(err.into()));
				}
			}
		}
		loop {
			match self.src.read_line(&mut body) {
				Ok(len) => {
					if len <= 1 {
						break;
					}
				}
				Err(err) => {
					return Some(Err(err.into()));
				}
			}
		}
		while field.ends_with('\n') {
			field.pop();
		}
		while body.ends_with('\n') {
			body.pop();
		}
		Some(Ok((field, body)))
	}
}

