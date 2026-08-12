//! Adapter for the ED6/ED7 scenario formats implemented by Themélios and
//! Calmare. This module intentionally keeps legacy types out of Kreuzen's
//! modern scenario model.

use calmare::Content;
use kreuzen::charmap::Charmap;
use themelios::scena::{ed6, ed7};
use themelios::types::Game as AureoleGame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
	Cp932,
	Gbk,
}

#[derive(Debug, Clone)]
pub struct TextCodec(cp932::Codec);

impl TextCodec {
	pub fn new(encoding: Encoding, charmap: &Charmap) -> Result<Self, Error> {
		let encoding = match encoding {
			Encoding::Cp932 => cp932::Encoding::Cp932,
			Encoding::Gbk => cp932::Encoding::Gbk,
		};
		let mut codec = cp932::Codec::new(encoding);
		for (bytes, glyph) in charmap.mappings() {
			codec
				.add_mapping(bytes.to_vec(), glyph)
				.map_err(|error| Error::Codec(error.to_string()))?;
		}
		Ok(Self(codec))
	}

	pub fn cp932() -> Self {
		Self(cp932::Codec::new(cp932::Encoding::Cp932))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineFamily {
	Ed6,
	Ed7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Game {
	SkyFc,
	SkyFcEvo,
	SkyFcKai,
	SkySc,
	SkyScEvo,
	SkyScKai,
	Sky3rd,
	Sky3rdEvo,
	Sky3rdKai,
	Zero,
	ZeroEvo,
	ZeroKai,
	Azure,
	AzureEvo,
	AzureKai,
}

impl Game {
	pub const fn engine(self) -> EngineFamily {
		match self {
			Self::SkyFc
			| Self::SkyFcEvo
			| Self::SkyFcKai
			| Self::SkySc
			| Self::SkyScEvo
			| Self::SkyScKai
			| Self::Sky3rd
			| Self::Sky3rdEvo
			| Self::Sky3rdKai => EngineFamily::Ed6,
			Self::Zero | Self::ZeroEvo | Self::ZeroKai | Self::Azure | Self::AzureEvo | Self::AzureKai => EngineFamily::Ed7,
		}
	}

	pub const fn binary_extension(self) -> &'static str {
		match self.engine() {
			EngineFamily::Ed6 if matches!(self, Self::SkyFc | Self::SkySc | Self::Sky3rd) => "_sn",
			EngineFamily::Ed6 | EngineFamily::Ed7 => "bin",
		}
	}

	const fn aureole(self) -> AureoleGame {
		match self {
			Self::SkyFc => AureoleGame::Fc,
			Self::SkyFcEvo => AureoleGame::FcEvo,
			Self::SkyFcKai => AureoleGame::FcKai,
			Self::SkySc => AureoleGame::Sc,
			Self::SkyScEvo => AureoleGame::ScEvo,
			Self::SkyScKai => AureoleGame::ScKai,
			Self::Sky3rd => AureoleGame::Tc,
			Self::Sky3rdEvo => AureoleGame::TcEvo,
			Self::Sky3rdKai => AureoleGame::TcKai,
			Self::Zero => AureoleGame::Zero,
			Self::ZeroEvo => AureoleGame::ZeroEvo,
			Self::ZeroKai => AureoleGame::ZeroKai,
			Self::Azure => AureoleGame::Ao,
			Self::AzureEvo => AureoleGame::AoEvo,
			Self::AzureKai => AureoleGame::AoKai,
		}
	}

	fn from_aureole(game: AureoleGame) -> Self {
		match game {
			AureoleGame::Fc => Self::SkyFc,
			AureoleGame::FcEvo => Self::SkyFcEvo,
			AureoleGame::FcKai => Self::SkyFcKai,
			AureoleGame::Sc => Self::SkySc,
			AureoleGame::ScEvo => Self::SkyScEvo,
			AureoleGame::ScKai => Self::SkyScKai,
			AureoleGame::Tc => Self::Sky3rd,
			AureoleGame::TcEvo => Self::Sky3rdEvo,
			AureoleGame::TcKai => Self::Sky3rdKai,
			AureoleGame::Zero => Self::Zero,
			AureoleGame::ZeroEvo => Self::ZeroEvo,
			AureoleGame::ZeroKai => Self::ZeroKai,
			AureoleGame::Ao => Self::Azure,
			AureoleGame::AoEvo => Self::AzureEvo,
			AureoleGame::AoKai => Self::AzureKai,
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("invalid text codec: {0}")]
	Codec(String),
	#[error("failed to read {game:?} scenario: {message}")]
	Read { game: Game, message: String },
	#[error("failed to parse legacy source: {0}")]
	Parse(String),
	#[error("failed to write {game:?} scenario: {message}")]
	Write { game: Game, message: String },
}

pub fn compile(source: &str, codec: &TextCodec) -> Result<(Game, Vec<u8>), Error> {
	cp932::with_codec(&codec.0, || compile_inner(source))
}

fn compile_inner(source: &str) -> Result<(Game, Vec<u8>), Error> {
	let (content, diagnostics) = calmare::parse(source, None);
	let fatal = diagnostics.iter().filter(|diagnostic| diagnostic.is_fatal()).collect::<Vec<_>>();
	if !fatal.is_empty() {
		return Err(Error::Parse(
			fatal.iter().map(|diagnostic| diagnostic.text.1.as_str()).collect::<Vec<_>>().join("; "),
		));
	}
	let Some((aureole, content)) = content else {
		return Err(Error::Parse("parser returned no content".to_owned()));
	};
	let game = Game::from_aureole(aureole);
	let bytes = match content {
		Content::ED6Scena(scena) => ed6::Scena::write(aureole, &scena),
		Content::ED7Scena(scena) => ed7::Scena::write(aureole, &scena),
	}
	.map_err(|error| Error::Write { game, message: error.to_string() })?;
	Ok((game, bytes))
}

pub fn compile_cp932(source: &str) -> Result<(Game, Vec<u8>), Error> {
	compile(source, &TextCodec::cp932())
}

/// Decompile one ED6/ED7 scenario using the legacy CP932 codec.
///
/// Explicit codec/charmap injection is introduced in P1. Keeping the codec
/// restriction visible here prevents the old process-global raw-byte switch
/// from becoming part of the new API.
pub fn decompile(game: Game, bytes: &[u8], codec: &TextCodec) -> Result<String, Error> {
	let aureole = game.aureole();
	cp932::with_codec(&codec.0, || {
		let content = match game.engine() {
			EngineFamily::Ed6 => {
				Content::ED6Scena(ed6::Scena::read(aureole, bytes).map_err(|error| Error::Read { game, message: error.to_string() })?)
			}
			EngineFamily::Ed7 => {
				Content::ED7Scena(ed7::Scena::read(aureole, bytes).map_err(|error| Error::Read { game, message: error.to_string() })?)
			}
		};
		Ok(calmare::to_string(aureole, &content, None))
	})
}

pub fn decompile_cp932(game: Game, bytes: &[u8]) -> Result<String, Error> {
	decompile(game, bytes, &TextCodec::cp932())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn profiles_route_to_the_expected_engine_family() {
		assert_eq!(Game::SkySc.engine(), EngineFamily::Ed6);
		assert_eq!(Game::SkySc.binary_extension(), "_sn");
		assert_eq!(Game::SkyScEvo.binary_extension(), "bin");
		assert_eq!(Game::ZeroKai.engine(), EngineFamily::Ed7);
		assert_eq!(Game::ZeroKai.binary_extension(), "bin");
	}

	#[test]
	fn builds_explicit_gbk_codec_with_charmap() {
		let map: Charmap = "FF40=♥".parse().unwrap();
		TextCodec::new(Encoding::Gbk, &map).unwrap();
	}
}
