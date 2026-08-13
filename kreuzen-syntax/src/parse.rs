mod alt;
mod parser;

use kreuzen::spec::Spec;
use kreuzen::{Enc, Scena, ScenaInfo};

use crate::diag::{Errors, Severity};
use crate::lex::{Cursor, Tokens};
pub use alt::TryParser;
pub use parser::{Error, Expect, Parser, Result};

/// Context available while parsing everything after the header.
#[derive(Clone, Copy)]
pub(crate) struct PCtx {
	pub spec: &'static Spec,
	pub can_break: bool,
	pub can_cont: bool,
}

/// Parses a `{}` block containing a statement-like sequence: items are
/// `;`-terminated unless they end with a `}` block, and a failed item skips
/// ahead and continues with the next one.
pub(crate) fn parse_block<T>(p: &mut Parser, mut f: impl FnMut(&mut Parser) -> Result<T>) -> Result<Vec<T>> {
	p.delim('{', |p| {
		let mut out = Vec::new();
		while !p.at_end() {
			parse_item(p, &mut out, &mut f);
		}
		Ok(out)
	})
}

/// A single item of a statement-like sequence; see [`parse_block`].
pub(crate) fn parse_item<T>(p: &mut Parser, out: &mut Vec<T>, f: impl FnOnce(&mut Parser) -> Result<T>) {
	let ok = match f(p) {
		Ok(v) => {
			out.push(v);
			p.cursor.prev_punct('}') || p.punct(';').is_ok()
		}
		// A failed item that ends on a `}` has recovered into its own block and
		// reported the error itself; see [`header_block`].
		Err(_) => p.cursor.prev_punct('}'),
	};
	if ok {
		// Whatever the item was still expecting is stale now.
		p.expect.clear();
	} else {
		p.report(recover);
	}
}

/// Parses a construct made of a header and a `{...}` block, such as `if`, `while` or `fn`.
///
/// If the header fails, the error is reported and the block is parsed anyway,
/// allowing the parser to collect errors inside, and followup blocks like `else`
/// and `shadow` can still be parsed afterwards.
pub(crate) fn header_block<H, B>(
	p: &mut Parser,
	header: impl FnOnce(&mut Parser) -> Result<H>,
	block: impl FnOnce(&mut Parser) -> Result<B>,
) -> Result<(H, B)> {
	match header(p) {
		Ok(h) => Ok((h, block(p)?)),
		Err(e) => {
			p.report(skip_to_block);
			if p.cursor.clone().delim('{').is_ok() {
				let _ = block(p);
			}
			Err(e)
		}
	}
}

/// Skips to just past the next `;` or `{...}` block, which is hopefully the end of the item.
pub(crate) fn recover(c: &mut Cursor) {
	while !c.at_end() {
		if c.punct(';').is_ok() || c.delim('{').is_ok() {
			break;
		}
		c.skip_any();
	}
}

/// Skips to the `{` of a block whose header failed, stopping short of a `;`,
/// which would mean the construct has no block after all.
fn skip_to_block(c: &mut Cursor) {
	while !c.at_end() && c.clone().punct(';').is_err() && c.clone().delim('{').is_err() {
		c.skip_any();
	}
}

/// The part of a file after the header, to be parsed with [`parse_scena`]
/// once the caller has picked an op table based on the header.
pub struct Rest<'a> {
	cursor: Cursor<'a>,
}

/// Phase 1: parses the `scena` header line.
pub fn parse_header<'a>(tokens: &'a Tokens, errors: &mut Errors) -> Option<(ScenaInfo, Rest<'a>)> {
	let mut p = Parser::new(tokens.cursor(), errors);
	match parse_header_inner(&mut p) {
		Ok(info) => Some((info, Rest { cursor: p.cursor })),
		Err(_) => {
			p.report(|_| {});
			None
		}
	}
}

fn parse_header_inner(p: &mut Parser) -> Result<ScenaInfo> {
	p.keyword("scena")?;
	let name = p.parse()?;

	let enc = p
		.alt()
		.test_kw("sjis", |_| Ok(Enc::Sjis))
		.test_kw("gbk", |_| Ok(Enc::Gbk))
		.test(|_| Ok(Enc::Utf8))
		.finish()?;
	let game = p.parse()?;
	let variant = if p.glued_punct('/').is_ok() { p.parse()? } else { 0 };
	let oddness = p.parse().unwrap_or(0);
	p.punct(';')?;

	Ok(ScenaInfo { name, game, enc, oddness, variant })
}

/// Phase 2: parses the rest of the file, using the given op table.
///
/// The spec must be `'static` because op names are borrowed from it;
/// get it from `kreuzen::spec::for_game`.
pub fn parse_scena(info: ScenaInfo, rest: Rest<'_>, spec: &'static Spec, errors: &mut Errors) -> Scena {
	let mut p = Parser::new(rest.cursor, errors);
	let ctx = PCtx { spec, can_break: false, can_cont: false };
	let mut chunks = Vec::new();
	while !p.at_end() {
		parse_item(&mut p, &mut chunks, |p| crate::scena::parse_chunk(p, &ctx));
	}
	Scena { info, chunks }
}

/// Convenience wrapper over both phases:
/// `parse(src, |info| spec::for_game(info.game, info.variant), &mut errors)`.
pub fn parse(src: &str, spec: impl FnOnce(&ScenaInfo) -> &'static Spec, errors: &mut Errors) -> Option<Scena> {
	let tokens = crate::lex::lex(src, errors);
	if errors.max_severity() >= Severity::Fatal {
		return None;
	}
	let (info, rest) = parse_header(&tokens, errors)?;
	let spec = spec(&info);
	Some(parse_scena(info, rest, spec, errors))
}

#[cfg(test)]
mod tests {
	use super::*;

	fn parse_errors(source: &str) -> Vec<String> {
		let mut errors = Errors::new();
		let scena = parse(source, |info| kreuzen::spec::for_game(info.game, info.variant), &mut errors);
		assert!(scena.is_some());
		errors.errors.into_iter().map(|error| error.main.desc).collect()
	}

	fn assert_recovers_inner_error(body: &str, marker: &str) {
		let source = format!("scena \"test\" cs1;\nfn \"test\" {{ {body} }}");
		let errors = parse_errors(&source);
		assert!(errors.iter().any(|error| error.contains(marker)), "missing {marker:?} in {errors:?}");
	}

	#[test]
	fn parses_gbk_header() {
		let mut errors = Errors::new();
		let tokens = crate::lex::lex("scena \"test\" gbk cs1;", &mut errors);
		let (info, _) = parse_header(&tokens, &mut errors).unwrap();
		assert_eq!(info.enc, Enc::Gbk);
		assert!(errors.is_empty());
	}

	#[test]
	fn malformed_if_header_still_checks_its_block() {
		assert_recovers_inner_error("if { break; }", "break outside");
	}

	#[test]
	fn malformed_while_header_still_checks_its_block() {
		assert_recovers_inner_error("while { bogus; }", "unknown op");
	}

	#[test]
	fn malformed_switch_header_still_checks_its_block() {
		assert_recovers_inner_error("switch { continue; }", "continue outside");
	}

	#[test]
	fn malformed_fork_lambda_header_still_checks_its_block() {
		assert_recovers_inner_error("ForkLambda { break; }", "break outside");
	}

	#[test]
	fn malformed_function_header_still_checks_its_block() {
		let errors = parse_errors("scena \"test\" cs1;\nfn { break; }");
		assert!(errors.iter().any(|error| error.contains("break outside")), "{errors:?}");
	}

	#[test]
	fn deprecated_opcode_alias_parses_with_warning() {
		let errors = parse_errors("scena \"test\" cs3;\nfn \"test\" { BondShow 0; }");
		assert_eq!(errors, ["deprecated op name: use 'BondShowExpGain'"]);
	}
}
