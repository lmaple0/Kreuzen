//! Adapter for the ED6/ED7 scenario formats implemented by Themélios and
//! Calmare. This module intentionally keeps legacy types out of Kreuzen's
//! modern scenario model.

use calmare::Content;
use themelios::scena::{ed6, ed7};
use themelios::types::Game as AureoleGame;

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
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("failed to read {game:?} scenario: {message}")]
	Read { game: Game, message: String },
}

/// Decompile one ED6/ED7 scenario using the legacy CP932 codec.
///
/// Explicit codec/charmap injection is introduced in P1. Keeping the codec
/// restriction visible here prevents the old process-global raw-byte switch
/// from becoming part of the new API.
pub fn decompile_cp932(game: Game, bytes: &[u8]) -> Result<String, Error> {
	let aureole = game.aureole();
	let content = match game.engine() {
		EngineFamily::Ed6 => Content::ED6Scena(ed6::Scena::read(aureole, bytes).map_err(|error| Error::Read { game, message: error.to_string() })?),
		EngineFamily::Ed7 => Content::ED7Scena(ed7::Scena::read(aureole, bytes).map_err(|error| Error::Read { game, message: error.to_string() })?),
	};
	Ok(calmare::to_string(aureole, &content, None))
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
}
