use std::cmp::Reverse;
use std::fmt;
use std::str::FromStr;

/// A bidirectional mapping between encoded bytes and the glyphs shown by a
/// modded or game-specific font.
///
/// The text format uses one `HEX=GLYPH` mapping per line. Empty lines and lines
/// starting with `#` are ignored. Each entry maps a prefix-free byte sequence
/// to exactly one Unicode scalar value.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Charmap {
	decode: Vec<(Vec<u8>, String)>,
	encode: Vec<(String, Vec<u8>)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharmapError {
	line: usize,
	message: String,
}

impl fmt::Display for CharmapError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "line {}: {}", self.line, self.message)
	}
}

impl std::error::Error for CharmapError {}

impl FromStr for Charmap {
	type Err = CharmapError;

	fn from_str(source: &str) -> Result<Self, Self::Err> {
		let mut decode = Vec::<(Vec<u8>, String)>::new();
		let mut encode = Vec::<(String, Vec<u8>)>::new();

		for (index, line) in source.lines().enumerate() {
			let line_number = index + 1;
			let line = line.trim().trim_start_matches('\u{FEFF}');
			if line.is_empty() || line.starts_with('#') {
				continue;
			}

			let Some((bytes, text)) = line.split_once('=') else {
				return Err(error(line_number, "expected HEX=TEXT"));
			};
			let bytes = bytes.trim().strip_prefix("0x").unwrap_or(bytes.trim());
			let bytes = hex::decode(bytes).map_err(|e| error(line_number, format!("invalid hex bytes: {e}")))?;
			let text = text.trim().to_owned();
			if bytes.is_empty() {
				return Err(error(line_number, "byte sequence must not be empty"));
			}
			if bytes.iter().any(|byte| *byte < 0x20) {
				return Err(error(line_number, "byte sequence must not contain text control bytes below 20"));
			}
			if text.is_empty() {
				return Err(error(line_number, "glyph must not be empty"));
			}
			if text.chars().count() != 1 {
				return Err(error(line_number, "right-hand side must be exactly one Unicode character"));
			}
			if text.contains('\0') {
				return Err(error(line_number, "glyph must not be NUL"));
			}
			if let Some((existing, _)) = decode
				.iter()
				.find(|(existing, _)| existing.starts_with(&bytes) || bytes.starts_with(existing))
			{
				return Err(error(
					line_number,
					format!(
						"byte sequence {} conflicts with prefix {}",
						hex::encode_upper(&bytes),
						hex::encode_upper(existing)
					),
				));
			}
			if encode.iter().any(|(existing, _)| existing == &text) {
				return Err(error(line_number, format!("duplicate text sequence {text:?}")));
			}

			decode.push((bytes.clone(), text.clone()));
			encode.push((text, bytes));
		}

		decode.sort_by_key(|entry| Reverse(entry.0.len()));
		encode.sort_by_key(|entry| Reverse(entry.0.len()));
		Ok(Self { decode, encode })
	}
}

impl Charmap {
	pub(crate) fn decode_match<'a>(&'a self, bytes: &[u8]) -> Option<(&'a str, usize)> {
		self.decode
			.iter()
			.find(|(encoded, _)| bytes.starts_with(encoded))
			.map(|(encoded, text)| (text.as_str(), encoded.len()))
	}

	pub(crate) fn encode_match<'a>(&'a self, text: &str) -> Option<(&'a [u8], usize)> {
		self.encode
			.iter()
			.find(|(decoded, _)| text.starts_with(decoded))
			.map(|(decoded, bytes)| (bytes.as_slice(), decoded.len()))
	}
}

fn error(line: usize, message: impl Into<String>) -> CharmapError {
	CharmapError { line, message: message.into() }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parses_glyph_entries() {
		let map: Charmap = "# comment\n8140=长\n".parse().unwrap();
		assert_eq!(map.decode_match(&[0x81, 0x40]), Some(("长", 2)));
		assert_eq!(map.encode_match("长文本"), Some((&[0x81, 0x40][..], "长".len())));
	}

	#[test]
	fn rejects_non_bijective_maps() {
		assert!(
			"81=甲\n81=乙"
				.parse::<Charmap>()
				.unwrap_err()
				.to_string()
				.contains("conflicts with prefix")
		);
		assert!("81=甲\n82=甲".parse::<Charmap>().unwrap_err().to_string().contains("duplicate text"));
		assert!(
			"81=甲\n8140=乙"
				.parse::<Charmap>()
				.unwrap_err()
				.to_string()
				.contains("conflicts with prefix")
		);
		assert!(
			"81=甲乙"
				.parse::<Charmap>()
				.unwrap_err()
				.to_string()
				.contains("exactly one Unicode character")
		);
	}
}
