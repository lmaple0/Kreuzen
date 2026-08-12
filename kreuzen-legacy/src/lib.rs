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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Layout {
	Native,
	#[default]
	Themelios,
}

impl Layout {
	fn aureole(self) -> ed7::Layout {
		match self {
			Self::Native => ed7::Layout::Native,
			Self::Themelios => ed7::Layout::Themelios,
		}
	}
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

pub fn compile(source: &str, codec: &TextCodec, layout: Layout) -> Result<(Game, Vec<u8>), Error> {
	let source = source.replace("\r\n", "\n");
	cp932::with_codec(&codec.0, || compile_inner(&source, layout))
}

fn compile_inner(source: &str, layout: Layout) -> Result<(Game, Vec<u8>), Error> {
	let (content, diagnostics) = calmare::parse(source, None);
	let fatal = diagnostics.iter().filter(|diagnostic| diagnostic.is_fatal()).collect::<Vec<_>>();
	if !fatal.is_empty() {
		let shown = fatal.iter().take(20).map(|diagnostic| {
			let offset = diagnostic.text.0.start.min(source.len());
			let prefix = &source[..offset];
			let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
			let column = prefix.rsplit_once('\n').map_or(prefix.len(), |(_, tail)| tail.len()) + 1;
			format!("{line}:{column}: {}", diagnostic.text.1)
		}).collect::<Vec<_>>();
		let omitted = fatal.len().saturating_sub(shown.len());
		let suffix = (omitted != 0).then(|| format!("; ... {omitted} more error(s)"));
		return Err(Error::Parse(shown.into_iter().chain(suffix).collect::<Vec<_>>().join("; ")));
	}
	let Some((aureole, content)) = content else {
		return Err(Error::Parse("parser returned no content".to_owned()));
	};
	let game = Game::from_aureole(aureole);
	let bytes = match content {
		Content::ED6Scena(scena) => ed6::Scena::write(aureole, &scena),
		Content::ED7Scena(scena) => ed7::Scena::write_with_layout(aureole, &scena, layout.aureole()),
	}
	.map_err(|error| Error::Write { game, message: error.to_string() })?;
	Ok((game, bytes))
}

pub fn compile_cp932(source: &str) -> Result<(Game, Vec<u8>), Error> {
	compile(source, &TextCodec::cp932(), Layout::Themelios)
}

/// Decompile one ED6/ED7 scenario using the legacy CP932 codec.
///
/// Explicit codec/charmap injection is introduced in P1. Keeping the codec
/// restriction visible here prevents the old process-global raw-byte switch
/// from becoming part of the new API.
pub fn decompile(game: Game, bytes: &[u8], codec: &TextCodec, layout: Layout) -> Result<String, Error> {
	let aureole = game.aureole();
	cp932::with_codec(&codec.0, || {
		let content = match game.engine() {
			EngineFamily::Ed6 => {
				Content::ED6Scena(ed6::Scena::read(aureole, bytes).map_err(|error| Error::Read { game, message: error.to_string() })?)
			}
			EngineFamily::Ed7 => {
				Content::ED7Scena(ed7::Scena::read_with_layout(aureole, bytes, layout.aureole()).map_err(|error| Error::Read {
					game,
					message: error.to_string(),
				})?)
			}
		};
		Ok(calmare::to_string(aureole, &content, None))
	})
}

pub fn decompile_cp932(game: Game, bytes: &[u8]) -> Result<String, Error> {
	decompile(game, bytes, &TextCodec::cp932(), Layout::Themelios)
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

	#[test]
	fn accepts_windows_line_endings() {
		let source = "calmare ao_k scena\r\nscena:\r\n\tname \"a\" \"b\" \"c\"\r\n";
		let error = compile(source, &TextCodec::cp932(), Layout::Themelios).unwrap_err();
		assert!(!error.to_string().contains("unexpected character"));
	}
}
