use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use kreuzen::{Enc, Game};
use kreuzen_legacy::{Encoding as LegacyEncoding, Game as LegacyGame, Layout as LegacyLayout, TextCodec};
use kreuzen_syntax::{Print as _, diag};
use rootcause::prelude::ResultExt as _;
use tracing_subscriber::prelude::*;
use walkdir::WalkDir;

#[derive(clap::ValueEnum, Clone, Copy)]
enum GameArg {
	Cs1,
	Cs2,
	Cs3,
	Cs4,
	Reverie,
	Tx,
	#[value(name = "sky-fc", alias = "fc")]
	SkyFc,
	#[value(name = "sky-fc-evo", alias = "fc_e")]
	SkyFcEvo,
	#[value(name = "sky-fc-kai", alias = "fc_k")]
	SkyFcKai,
	#[value(name = "sky-sc", alias = "sc")]
	SkySc,
	#[value(name = "sky-sc-evo", alias = "sc_e")]
	SkyScEvo,
	#[value(name = "sky-sc-kai", alias = "sc_k")]
	SkyScKai,
	#[value(name = "sky-3rd", alias = "tc")]
	Sky3rd,
	#[value(name = "sky-3rd-evo", alias = "tc_e")]
	Sky3rdEvo,
	#[value(name = "sky-3rd-kai", alias = "tc_k")]
	Sky3rdKai,
	Zero,
	#[value(name = "zero-evo", alias = "zero_e")]
	ZeroEvo,
	#[value(name = "zero-kai", alias = "zero_k")]
	ZeroKai,
	#[value(name = "azure", alias = "ao")]
	Azure,
	#[value(name = "azure-evo", alias = "ao_e")]
	AzureEvo,
	#[value(name = "azure-kai", alias = "ao_k")]
	AzureKai,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameProfile {
	Modern(Game),
	Legacy(LegacyGame),
}

impl From<GameArg> for GameProfile {
	fn from(g: GameArg) -> Self {
		match g {
			GameArg::Cs1 => Self::Modern(Game::Cs1),
			GameArg::Cs2 => Self::Modern(Game::Cs2),
			GameArg::Cs3 => Self::Modern(Game::Cs3),
			GameArg::Cs4 => Self::Modern(Game::Cs4),
			GameArg::Reverie => Self::Modern(Game::Reverie),
			GameArg::Tx => Self::Modern(Game::Tx),
			GameArg::SkyFc => Self::Legacy(LegacyGame::SkyFc),
			GameArg::SkyFcEvo => Self::Legacy(LegacyGame::SkyFcEvo),
			GameArg::SkyFcKai => Self::Legacy(LegacyGame::SkyFcKai),
			GameArg::SkySc => Self::Legacy(LegacyGame::SkySc),
			GameArg::SkyScEvo => Self::Legacy(LegacyGame::SkyScEvo),
			GameArg::SkyScKai => Self::Legacy(LegacyGame::SkyScKai),
			GameArg::Sky3rd => Self::Legacy(LegacyGame::Sky3rd),
			GameArg::Sky3rdEvo => Self::Legacy(LegacyGame::Sky3rdEvo),
			GameArg::Sky3rdKai => Self::Legacy(LegacyGame::Sky3rdKai),
			GameArg::Zero => Self::Legacy(LegacyGame::Zero),
			GameArg::ZeroEvo => Self::Legacy(LegacyGame::ZeroEvo),
			GameArg::ZeroKai => Self::Legacy(LegacyGame::ZeroKai),
			GameArg::Azure => Self::Legacy(LegacyGame::Azure),
			GameArg::AzureEvo => Self::Legacy(LegacyGame::AzureEvo),
			GameArg::AzureKai => Self::Legacy(LegacyGame::AzureKai),
		}
	}
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum EncArg {
	Utf8,
	Sjis,
	Gbk,
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum DecompileMode {
	Flat,
	Tree,
	Sugar,
}

#[derive(clap::ValueEnum, Clone, Copy, Default)]
enum LegacyLayoutArg {
	Native,
	#[default]
	Themelios,
}

impl From<LegacyLayoutArg> for LegacyLayout {
	fn from(layout: LegacyLayoutArg) -> Self {
		match layout {
			LegacyLayoutArg::Native => Self::Native,
			LegacyLayoutArg::Themelios => Self::Themelios,
		}
	}
}

#[derive(clap::Parser)]
#[command(
	about = "Falcom scenario compiler/decompiler with modern and ED6/ED7 backends",
	long_about = "Falcom scenario compiler/decompiler with modern and ED6/ED7 backends.\n\nSky SC and Sky the 3rd PC scripts use ._SN <-> .clm with --game sky-sc or --game sky-3rd. Chinese patches normally require explicit --enc gbk. Legacy encoding and ED7 layout failures are reported directly; Kreuzen never retries another value automatically."
)]
struct Args {
	#[arg(value_name = "FILES", help = "Scenario/source files or one directory")]
	files: Vec<PathBuf>,

	#[clap(long, help = "Source game; explicit value overrides path/executable detection")]
	game: Option<GameArg>,
	#[clap(long, help = "Script text encoding; legacy default is sjis, Chinese patches normally use gbk")]
	enc: Option<EncArg>,
	#[clap(long, value_name = "FILE", help = "Custom HEX=GLYPH character map")]
	charmap: Option<PathBuf>,
	#[clap(long, default_value = "sugar", help = "Modern-backend decompile depth")]
	mode: DecompileMode,
	#[clap(long, default_value = "themelios", help = "ED7-only binary layout; never auto-detected or retried")]
	legacy_layout: LegacyLayoutArg,
	#[clap(long, short, help = "Output file")]
	output: Option<PathBuf>,
}

impl Args {
	fn game_for(&self, path: &Path) -> Option<GameProfile> {
		self.game.map(GameProfile::from).or_else(|| detect_game(path))
	}
}

fn main() -> ExitCode {
	tracing_subscriber::registry()
		.with(
			tracing_subscriber::fmt::layer()
				.with_writer(std::io::stderr)
				.with_ansi_sanitization(false),
		)
		.with(
			tracing_subscriber::EnvFilter::builder()
				.with_default_directive(tracing::Level::INFO.into())
				.from_env_lossy(),
		)
		.init();
	let args = Args::parse();

	let mut success = true;

	if args.output.is_some() && args.files.len() > 1 {
		tracing::error!("Cannot specify output file with multiple input files");
		success = false;
	} else {
		for path in &args.files {
			let _span = tracing::debug_span!("process_arg", path = %path.display()).entered();
			if !path.exists() {
				tracing::error!("File does not exist: {}", path.display());
				success = false;
			} else if path.is_dir() {
				success &= handle_dir(&args, path, args.output.as_deref());
			} else {
				success &= handle_file(&args, path, args.output.as_deref());
			}
		}
	}

	if success {
		ExitCode::SUCCESS
	} else {
		windows_wait();
		ExitCode::FAILURE
	}
}

#[cfg(target_os = "windows")]
fn windows_wait() {
	use windows_sys::Win32::System::Console::{GetConsoleProcessList, GetConsoleWindow};
	if unsafe { GetConsoleWindow() }.is_null() {
		return;
	}
	let process_count: u32 = unsafe { GetConsoleProcessList([0].as_mut_ptr(), 1) };
	if process_count == 1 {
		std::process::Command::new("cmd").arg("/c").arg("pause").status().ok();
	}
}

#[cfg(not(target_os = "windows"))]
fn windows_wait() {}

fn handle_dir(args: &Args, path: &Path, out: Option<&Path>) -> bool {
	if matches!(args.game_for(path), Some(GameProfile::Legacy(_))) {
		return handle_legacy_dir(args, path, out);
	}

	let mut krz = Vec::new();
	let mut dat = Vec::new();
	for entry in WalkDir::new(path).into_iter().filter_map(|v| v.ok()) {
		if entry.metadata().is_ok_and(|m| m.is_file()) {
			if entry.path().extension().is_some_and(|e| e == "krz") {
				krz.push(entry.path().strip_prefix(path).unwrap().to_owned());
			} else if entry.path().extension().is_some_and(|e| e == "dat") && !skip_dat(args, entry.path()) {
				dat.push(entry.path().strip_prefix(path).unwrap().to_owned());
			}
		}
	}

	if !krz.is_empty() && !dat.is_empty() {
		tracing::error!(
			"Found both krz ({}) and dat ({}) files in the same directory",
			krz[0].display(),
			dat[0].display()
		);
		false
	} else if !krz.is_empty() {
		let outdir = out_dir(path, out, ".krz", ".dat");
		let mut success = true;
		for file in krz {
			let infile = path.join(&file);
			let outfile = out_file(&outdir.join(&file), ".krz", ".dat");
			success &= compile(args, &infile, &outfile);
		}
		success
	} else if !dat.is_empty() {
		let outdir = out_dir(path, out, ".dat", ".krz");
		let mut success = true;
		for file in dat {
			let infile = path.join(&file);
			let outfile = out_file(&outdir.join(&file), ".dat", ".krz");
			success &= decompile(args, &infile, &outfile);
		}
		success
	} else {
		tracing::error!("No krz or dat files found in directory");
		false
	}
}

fn handle_legacy_dir(args: &Args, path: &Path, out: Option<&Path>) -> bool {
	let Some(GameProfile::Legacy(game)) = args.game_for(path) else {
		tracing::error!("Specify a legacy --game when processing an ED6/ED7 directory");
		return false;
	};
	let mut source = Vec::new();
	let mut binary = Vec::new();
	for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
		if !entry.metadata().is_ok_and(|metadata| metadata.is_file()) {
			continue;
		}
		let extension = entry.path().extension().and_then(|value| value.to_str()).unwrap_or_default();
		let relative = entry.path().strip_prefix(path).expect("walked path is below root").to_owned();
		if extension.eq_ignore_ascii_case("clm") {
			source.push(relative);
		} else if extension.eq_ignore_ascii_case(game.binary_extension()) {
			binary.push(relative);
		}
	}

	if !source.is_empty() && !binary.is_empty() {
		tracing::error!(
			"Found both clm ({}) and binary ({}) files in the same directory",
			source[0].display(),
			binary[0].display()
		);
		return false;
	}
	if source.is_empty() && binary.is_empty() {
		tracing::error!("No clm or {} files found in directory", game.binary_extension());
		return false;
	}

	if !source.is_empty() {
		let outdir = out
			.map(Path::to_owned)
			.unwrap_or_else(|| path.with_file_name(format!("{}.bin", path.file_name().unwrap().to_string_lossy())));
		return source.into_iter().fold(true, |success, relative| {
			let infile = path.join(&relative);
			let outfile = outdir.join(relative.with_extension(game.binary_extension()));
			success & compile_legacy(args, &infile, Some(&outfile))
		});
	}

	let outdir = out
		.map(Path::to_owned)
		.unwrap_or_else(|| path.with_file_name(format!("{}.clm", path.file_name().unwrap().to_string_lossy())));
	binary.into_iter().fold(true, |success, relative| {
		let infile = path.join(&relative);
		let outfile = outdir.join(relative.with_extension("clm"));
		success & decompile(args, &infile, &outfile)
	})
}

fn handle_file(args: &Args, path: &Path, out: Option<&Path>) -> bool {
	if path.extension().is_some_and(|e| e == "krz") {
		let infile = path;
		let outfile = resolve_out(out, path, ".krz", ".dat");
		compile(args, infile, &outfile)
	} else if path.extension().is_some_and(|e| e == "dat") {
		let infile = path;
		let outfile = resolve_out(out, path, ".dat", ".krz");
		decompile(args, infile, &outfile)
	} else if path
		.extension()
		.is_some_and(|e| e.eq_ignore_ascii_case("bin") || e.eq_ignore_ascii_case("_sn"))
	{
		let infile = path;
		let outfile = path.with_extension("clm");
		let outfile = out.map_or(outfile, Path::to_owned);
		decompile(args, infile, &outfile)
	} else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("clm")) {
		compile_legacy(args, path, out)
	} else {
		tracing::error!("File is not a supported scenario or source file");
		false
	}
}

/// Resolve the output path for a single file. When no output is given, the file
/// is written next to the input with a swapped suffix. When the output is an
/// existing directory, the derived filename is placed inside it.
fn resolve_out(out: Option<&Path>, path: &Path, old_suffix: &str, new_suffix: &str) -> PathBuf {
	match out {
		None => out_file(path, old_suffix, new_suffix),
		Some(out) if out.is_dir() => out.join(out_file(path, old_suffix, new_suffix).file_name().unwrap()),
		Some(out) => out.to_owned(),
	}
}

fn out_file(path: &Path, old_suffix: &str, new_suffix: &str) -> PathBuf {
	let name = path.file_name().unwrap().to_str().unwrap();
	let name = name.strip_suffix(old_suffix).expect("suffix is already checked");
	path.with_file_name(format!("{name}{new_suffix}"))
}

fn out_dir(path: &Path, out: Option<&Path>, old_suffix: &str, new_suffix: &str) -> PathBuf {
	if let Some(out) = out {
		return out.to_owned();
	}
	let name = path.file_name().unwrap().to_str().unwrap();
	if let Some(name) = name.strip_suffix(old_suffix) {
		path.with_file_name(name)
	} else {
		path.with_file_name(format!("{name}{new_suffix}"))
	}
}

fn decompile(args: &Args, infile: &Path, outfile: &Path) -> bool {
	let _span = tracing::error_span!("decompile", file = %infile.display()).entered();
	match decompile_inner(args, infile, outfile) {
		Ok(v) => v,
		Err(e) => {
			tracing::error!("{e}");
			tracing::error!("This is probably a bug in Kreuzen, please report it.");
			false
		}
	}
}

fn compile(args: &Args, infile: &Path, outfile: &Path) -> bool {
	let _span = tracing::error_span!("compile", file = %infile.display()).entered();
	match compile_inner(args, infile, outfile) {
		Ok(v) => v,
		Err(e) => {
			tracing::error!("{e}");
			tracing::error!("This is probably a bug in Kreuzen, please report it.");
			false
		}
	}
}

fn compile_legacy(args: &Args, infile: &Path, out: Option<&Path>) -> bool {
	let _span = tracing::error_span!("compile_legacy", file = %infile.display()).entered();
	match compile_legacy_inner(args, infile, out) {
		Ok(v) => v,
		Err(e) => {
			tracing::error!("{e}");
			false
		}
	}
}

fn compile_legacy_inner(args: &Args, infile: &Path, out: Option<&Path>) -> rootcause::Result<bool> {
	let source = std::fs::read_to_string(infile).context_with(|| format!("failed to read file: {}", infile.display()))?;
	let codec = legacy_codec(args)?;
	let (game, bytes) = kreuzen_legacy::compile(&source, &codec, args.legacy_layout.into()).map_err(|error| rootcause::report!("{error}"))?;
	if let Some(expected) = args.game.map(GameProfile::from)
		&& expected != GameProfile::Legacy(game)
	{
		rootcause::bail!("source declares {game:?}, but --game selects a different profile");
	}
	let derived = infile.with_extension(game.binary_extension());
	let outfile = match out {
		Some(out) if out.is_dir() => out.join(derived.file_name().unwrap()),
		Some(out) => out.to_owned(),
		None => derived,
	};
	write_file(&outfile, &bytes)?;
	Ok(true)
}

fn decompile_inner(args: &Args, infile: &Path, outfile: &Path) -> rootcause::Result<bool> {
	let Some(game) = args.game_for(infile) else {
		tracing::error!("Could not detect game from exe name or path; specify --game to decompile");
		return Ok(false);
	};
	let bytes = std::fs::read(infile).context_with(|| format!("failed to read file: {}", infile.display()))?;
	let str = match game {
		GameProfile::Modern(game) => {
			let enc = match args.enc {
				Some(EncArg::Utf8) => Enc::Utf8,
				Some(EncArg::Sjis) => Enc::Sjis,
				Some(EncArg::Gbk) => Enc::Gbk,
				None => detect_enc(game, infile),
			};
			let charmap = load_charmap(args)?;
			let scena = kreuzen::read_with_charmap(game, enc, &bytes, &charmap).context("failed to read scena")?;
			let scena = match args.mode {
				DecompileMode::Flat => scena,
				DecompileMode::Tree => kreuzen::decompile(&scena)?,
				DecompileMode::Sugar => kreuzen::sugar::resugar(&kreuzen::decompile(&scena)?)?,
			};
			scena.print_to_string()
		}
		GameProfile::Legacy(game) => {
			let codec = legacy_codec(args)?;
			kreuzen_legacy::decompile(game, &bytes, &codec, args.legacy_layout.into()).map_err(|error| rootcause::report!("{error}"))?
		}
	};
	write_file(outfile, str.as_bytes())?;
	Ok(true)
}

fn compile_inner(args: &Args, infile: &Path, outfile: &Path) -> rootcause::Result<bool> {
	let source = std::fs::read_to_string(infile).context_with(|| format!("failed to read file: {}", infile.display()))?;

	let mut errors = diag::Errors::new();
	let scena = kreuzen_syntax::parse(&source, |i| kreuzen::spec::for_game(i.game, i.variant), &mut errors);
	if !errors.is_empty() {
		print!("{}", diag::render(&infile.display().to_string(), &source, &errors));
	}
	if errors.max_severity() >= diag::Severity::Error {
		return Ok(false);
	}
	let Some(scena) = scena else {
		return Ok(false);
	};

	let scena = kreuzen::sugar::desugar(&scena)?;
	let scena = kreuzen::compile(&scena)?;
	let charmap = load_charmap(args)?;
	let data = kreuzen::write_with_charmap(&scena, &charmap).context("failed to write scena")?;
	write_file(outfile, &data)?;
	Ok(true)
}

fn skip_dat(args: &Args, path: &Path) -> bool {
	match path.file_name().and_then(|n| n.to_str()) {
		Some("utf8sjis.dat" | "sjisutf8.dat") => true,
		Some("magic.dat") => args.game_for(path) == Some(GameProfile::Modern(Game::Tx)),
		_ => false,
	}
}

fn detect_game(infile: &Path) -> Option<GameProfile> {
	detect_game_from_exe().or_else(|| parents(infile).find_map(detect_game_from_install_component))
}

fn detect_game_from_install_component(component: &str) -> Option<GameProfile> {
	let profiles = [
		("Trails of Cold Steel", GameProfile::Modern(Game::Cs1)),
		("Trails of Cold Steel II", GameProfile::Modern(Game::Cs2)),
		("The Legend of Heroes Trails of Cold Steel III", GameProfile::Modern(Game::Cs3)),
		("The Legend of Heroes Trails of Cold Steel IV", GameProfile::Modern(Game::Cs4)),
		("The Legend of Heroes Trails into Reverie", GameProfile::Modern(Game::Reverie)),
		("Tokyo Xanadu eX+", GameProfile::Modern(Game::Tx)),
		("Trails in the Sky", GameProfile::Legacy(LegacyGame::SkyFc)),
		("Trails in the Sky SC", GameProfile::Legacy(LegacyGame::SkySc)),
		("Trails in the Sky the 3rd", GameProfile::Legacy(LegacyGame::Sky3rd)),
		("The Legend of Heroes Trails from Zero", GameProfile::Legacy(LegacyGame::ZeroKai)),
		("The Legend of Heroes Trails to Azure", GameProfile::Legacy(LegacyGame::AzureKai)),
	];
	profiles
		.into_iter()
		.find_map(|(name, profile)| component.eq_ignore_ascii_case(name).then_some(profile))
}

fn detect_game_from_exe() -> Option<GameProfile> {
	let exe = std::env::current_exe().ok()?.file_stem()?.to_str()?.to_ascii_lowercase();
	match exe.as_str() {
		"kreuzen-cs1" => Some(GameProfile::Modern(Game::Cs1)),
		"kreuzen-cs2" => Some(GameProfile::Modern(Game::Cs2)),
		"kreuzen-cs3" => Some(GameProfile::Modern(Game::Cs3)),
		"kreuzen-cs4" => Some(GameProfile::Modern(Game::Cs4)),
		"kreuzen-reverie" => Some(GameProfile::Modern(Game::Reverie)),
		"kreuzen-tx" => Some(GameProfile::Modern(Game::Tx)),
		"kreuzen-sky-fc" => Some(GameProfile::Legacy(LegacyGame::SkyFc)),
		"kreuzen-sky-sc" => Some(GameProfile::Legacy(LegacyGame::SkySc)),
		"kreuzen-sky-3rd" => Some(GameProfile::Legacy(LegacyGame::Sky3rd)),
		"kreuzen-zero" => Some(GameProfile::Legacy(LegacyGame::ZeroKai)),
		"kreuzen-azure" => Some(GameProfile::Legacy(LegacyGame::AzureKai)),
		_ => None,
	}
}

/// For Cs1/Cs2, picks sjis or utf8 based on a `dat`/`dat_us` ancestor folder;
/// every other game is always utf8.
fn detect_enc(game: Game, infile: &Path) -> Enc {
	if matches!(game, Game::Cs1 | Game::Cs2) {
		for c in parents(infile) {
			match c {
				"dat" => return Enc::Sjis,
				"dat_us" => return Enc::Utf8,
				_ => {}
			}
		}
	}
	// Probably a sensible default for the cs1/2 scripts as well;
	// most modders are probably interested in the english ones.
	Enc::Utf8
}

fn parents(path: &Path) -> impl Iterator<Item = &str> {
	path.parent()
		.into_iter()
		.flat_map(|p| p.components())
		.filter_map(|c| c.as_os_str().to_str())
}

fn load_charmap(args: &Args) -> rootcause::Result<kreuzen::charmap::Charmap> {
	let Some(path) = &args.charmap else {
		return Ok(kreuzen::charmap::Charmap::default());
	};
	let source = std::fs::read_to_string(path).context_with(|| format!("failed to read charmap: {}", path.display()))?;
	source.parse().map_err(|e| rootcause::report!("invalid charmap {}: {e}", path.display()))
}

fn legacy_codec(args: &Args) -> rootcause::Result<TextCodec> {
	let encoding = match args.enc.unwrap_or(EncArg::Sjis) {
		EncArg::Sjis => LegacyEncoding::Cp932,
		EncArg::Gbk => LegacyEncoding::Gbk,
		EncArg::Utf8 => rootcause::bail!("UTF-8 is not a valid ED6/ED7 binary text encoding"),
	};
	let charmap = load_charmap(args)?;
	TextCodec::new(encoding, &charmap).map_err(|error| rootcause::report!("{error}"))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn detects_legacy_games_from_install_paths() {
		assert_eq!(
			detect_game(Path::new(r"C:\Games\Trails in the Sky SC\DAT\ED6_DT21\A0019._SN")),
			Some(GameProfile::Legacy(LegacyGame::SkySc))
		);
		assert_eq!(
			detect_game(Path::new(r"C:\Games\TRAILS IN THE SKY THE 3RD\data\ED6_DT21\a0028._sn")),
			Some(GameProfile::Legacy(LegacyGame::Sky3rd))
		);
		assert_eq!(
			detect_game(Path::new(r"C:\Games\The Legend of Heroes Trails from Zero\data\scena\a0003.bin")),
			Some(GameProfile::Legacy(LegacyGame::ZeroKai))
		);
	}
}

fn write_file(outfile: &Path, data: &[u8]) -> rootcause::Result<()> {
	if let Some(parent) = outfile.parent()
		&& !parent.exists()
	{
		std::fs::create_dir_all(parent).context_with(|| format!("failed to create directory: {}", parent.display()))?;
	}
	std::fs::write(outfile, data).context_with(|| format!("failed to write file: {}", outfile.display()))?;
	Ok(())
}
