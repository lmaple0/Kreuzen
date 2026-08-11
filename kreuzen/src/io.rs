use std::borrow::Cow;

use crate::charmap::Charmap;
use crate::{Enc, Game};
use gospel::read::Reader;
use gospel::write::{Label, Writer};

pub(crate) fn encode(enc: Enc, s: &str, charmap: &Charmap) -> rootcause::Result<Vec<u8>> {
	let mut out = Vec::new();
	let mut position = 0;
	let mut unmapped = 0;
	while position < s.len() {
		if let Some((bytes, len)) = charmap.encode_match(&s[position..]) {
			out.extend(encode_base(enc, &s[unmapped..position], charmap)?);
			out.extend(bytes);
			position += len;
			unmapped = position;
		} else {
			position += s[position..].chars().next().unwrap().len_utf8();
		}
	}
	out.extend(encode_base(enc, &s[unmapped..], charmap)?);
	Ok(out)
}

fn encode_base(enc: Enc, s: &str, charmap: &Charmap) -> rootcause::Result<Vec<u8>> {
	let bytes = match enc {
		Enc::Utf8 => s.as_bytes().to_vec(),
		Enc::Sjis => match falcom_sjis::encode(&if charmap.decode_match(&[0x87, 0x8A]).is_none() {
			s.replace('♥', "㈱")
		} else {
			s.to_owned()
		}) {
			Ok(bytes) => bytes,
			Err(pos) => rootcause::bail!("invalid Shift-JIS at byte {pos}: {s:?}"),
		},
		Enc::Gbk => {
			let (bytes, _, had_errors) = encoding_rs::GBK.encode(s);
			if had_errors {
				rootcause::bail!("invalid GBK text: {s:?}")
			}
			bytes.into_owned()
		}
	};
	if (0..bytes.len()).any(|position| charmap.decode_match(&bytes[position..]).is_some()) {
		rootcause::bail!("text encodes to bytes reserved by the charmap: {s:?}");
	}
	Ok(bytes)
}

fn decode(enc: Enc, bytes: &[u8], charmap: &Charmap) -> rootcause::Result<String> {
	let mut out = String::new();
	let mut position = 0;
	let mut unmapped = 0;
	while position < bytes.len() {
		if let Some((text, len)) = charmap.decode_match(&bytes[position..]) {
			out.push_str(&decode_base(enc, &bytes[unmapped..position])?);
			out.push_str(text);
			position += len;
			unmapped = position;
		} else {
			position += encoded_char_len(enc, &bytes[position..])?;
		}
	}
	out.push_str(&decode_base(enc, &bytes[unmapped..])?);
	Ok(out)
}

fn decode_base(enc: Enc, bytes: &[u8]) -> rootcause::Result<String> {
	match enc {
		Enc::Utf8 => match String::from_utf8_lossy(bytes) {
			Cow::Borrowed(text) => Ok(text.to_owned()),
			Cow::Owned(e) => {
				if let Ok(mut s) = falcom_sjis::decode(bytes) {
					tracing::warn!("Invalid UTF-8 in text, but valid Shift-JIS: {s:?}");
					s.insert(0, '\u{FFFD}');
					Ok(s)
				} else {
					rootcause::bail!("Invalid UTF-8 in text: {e:?}");
				}
			}
		},
		Enc::Sjis => match falcom_sjis::decode(bytes) {
			Ok(text) => Ok(text.replace('㈱', "♥")),
			Err(_) => rootcause::bail!("Invalid Shift-JIS in text: {e:?}", e = falcom_sjis::decode_lossy(bytes)),
		},
		Enc::Gbk => match encoding_rs::GBK.decode_without_bom_handling_and_without_replacement(bytes) {
			Some(text) => Ok(text.into_owned()),
			None => rootcause::bail!("Invalid GBK in text: {:02X?}", bytes),
		},
	}
}

fn encoded_char_len(enc: Enc, bytes: &[u8]) -> rootcause::Result<usize> {
	let Some(&first) = bytes.first() else {
		return Ok(0);
	};
	let len = match enc {
		Enc::Utf8 => match first {
			..=0x7F => 1,
			0xC2..=0xDF => 2,
			0xE0..=0xEF => 3,
			0xF0..=0xF4 => 4,
			_ => rootcause::bail!("Invalid UTF-8 lead byte: {first:02X}"),
		},
		Enc::Sjis => match first {
			0x00..=0x7F | 0xA1..=0xDF => 1,
			_ => 2,
		},
		Enc::Gbk => match first {
			0x00..=0x80 => 1,
			_ => 2,
		},
	};
	let Some(char_bytes) = bytes.get(..len) else {
		rootcause::bail!("Truncated {:?} character: {:02X?}", enc, bytes);
	};
	decode_base(enc, char_bytes)?;
	Ok(len)
}

#[derive(Debug, derive_more::Deref, derive_more::DerefMut)]
pub struct CReader<'a> {
	pub game: Game,
	pub enc: Enc,
	pub charmap: &'a Charmap,
	pub scena: &'a str,
	pub variant: u8,
	pub outline_start: usize,
	#[deref]
	#[deref_mut]
	pub reader: Reader<'a>,
}

impl<'a> CReader<'a> {
	#[track_caller]
	pub fn str(&mut self) -> rootcause::Result<String> {
		let cstr = self.cstr()?;
		let s = self.decode(cstr.to_bytes())?;
		Ok(s)
	}

	#[track_caller]
	pub fn sstr(&mut self, s: usize) -> rootcause::Result<String> {
		let pos = self.pos();
		let str = self.slice(s)?;
		let len = str.iter().position(|&b| b == 0).unwrap_or(s);
		let cstr = &str[..len];
		let s = self.decode(cstr)?;
		if !str[len..].iter().all(|&b| b == 0) {
			rootcause::bail!("Nonzero padding on sized string at {pos:X}: {s:?}");
		}
		Ok(s)
	}

	#[track_caller]
	pub fn decode(&self, bytes: &[u8]) -> rootcause::Result<String> {
		decode(self.enc, bytes, self.charmap)
	}

	pub fn rewind(&mut self) {
		self.reader.seek(self.reader.pos() - 1).ok();
	}
}

pub struct OData<'a> {
	pub start: Label,
	pub game: Game,
	pub enc: Enc,
	pub charmap: &'a Charmap,
	pub variant: u8,
}

pub trait WriterExt {
	fn str(&mut self, enc: Enc, charmap: &Charmap, s: &str) -> rootcause::Result<()>;
	fn sstr(&mut self, len: usize, enc: Enc, charmap: &Charmap, s: &str) -> rootcause::Result<()>;
}

impl WriterExt for Writer {
	fn str(&mut self, enc: Enc, charmap: &Charmap, s: &str) -> rootcause::Result<()> {
		let bytes = encode(enc, s, charmap)?;
		if bytes.contains(&0) {
			rootcause::bail!("string contains NUL: {s:?}");
		}
		self.slice(&bytes);
		self.slice(&[0]);
		Ok(())
	}

	fn sstr(&mut self, len: usize, enc: Enc, charmap: &Charmap, s: &str) -> rootcause::Result<()> {
		let bytes = encode(enc, s, charmap)?;
		if bytes.contains(&0) {
			rootcause::bail!("string contains NUL: {s:?}");
		}
		if bytes.len() > len {
			rootcause::bail!("string too long for sstr({len}): {s:?} encodes to {} bytes", bytes.len());
		}
		self.slice(&bytes);
		self.slice(&vec![0; len - bytes.len()]);
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn gbk_roundtrip() {
		let charmap = Charmap::default();
		let encoded = encode(Enc::Gbk, "中文测试", &charmap).unwrap();
		assert_eq!(encoded, hex::decode("D6D0CEC4B2E2CAD4").unwrap());
		assert_eq!(decode(Enc::Gbk, &encoded, &charmap).unwrap(), "中文测试");
	}

	#[test]
	fn falcom_heart_mapping_roundtrip() {
		let charmap = Charmap::default();
		let encoded = encode(Enc::Sjis, "♥", &charmap).unwrap();
		assert_eq!(encoded, hex::decode("878A").unwrap());
		assert_eq!(decode(Enc::Sjis, &encoded, &charmap).unwrap(), "♥");
	}

	#[test]
	fn custom_charmap_bypasses_base_encoding() {
		let charmap: Charmap = "FF=á\nF040=ế\nF041=ệ\n".parse().unwrap();
		let encoded = encode(Enc::Sjis, "á Tiếng Việt", &charmap).unwrap();
		assert_eq!(
			encoded,
			[vec![0xFF], b" Ti".to_vec(), vec![0xF0, 0x40], b"ng Vi".to_vec(), vec![0xF0, 0x41, b't']].concat()
		);
		assert_eq!(decode(Enc::Sjis, &encoded, &charmap).unwrap(), "á Tiếng Việt");
	}

	#[test]
	fn custom_charmap_rejects_base_encoded_collisions() {
		let charmap: Charmap = "41=Ｘ\n".parse().unwrap();
		assert!(
			encode(Enc::Sjis, "A", &charmap)
				.unwrap_err()
				.to_string()
				.contains("reserved by the charmap")
		);
	}

	#[test]
	fn custom_charmap_can_override_falcom_heart_mapping() {
		let charmap: Charmap = "878A=☆\n".parse().unwrap();
		assert_eq!(decode(Enc::Sjis, &[0x87, 0x8A], &charmap).unwrap(), "☆");
		assert!(encode(Enc::Sjis, "♥", &charmap).is_err());
	}
}
