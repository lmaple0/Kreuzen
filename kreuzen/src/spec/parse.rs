use rootcause::option_ext::OptionExt as _;
use rootcause::prelude::{IteratorExt as _, ResultExt as _};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use crate::spec::{Op, Opcode, Part, Spec};
use crate::Game;

pub type Lines = BTreeMap<Opcode, Line>;

#[derive(Debug, Clone, Default)]
pub struct Line {
	pub name: String,
	/// Former names of this op, accepted when parsing but never printed.
	pub aliases: Vec<String>,
	pub parts: Vec<Part>,
}

pub fn parse_lines(name: &str) -> Lines {
	match try_parse_lines(name) {
		Ok(lines) => lines,
		Err(e) => {
			eprintln!("{e}");
			std::process::exit(1);
		}
	}
}

pub fn try_parse_lines(name: &str) -> rootcause::Result<Lines> {
	let text = super::text_for(name).context_with(|| format!("unknown spec: {name}"))?;
	let mut ops = BTreeMap::new();
	let mut add = |code: Opcode, line: Line| {
		assert!(!ops.contains_key(&code), "Duplicate code in spec: {code} and {}", line.name);
		ops.insert(code, line);
	};
	() = text
		.lines()
		.map(|line| parse_line(line, &mut add).context_with(|| format!("error parsing line: {line:?}")))
		.collect_reports()
		.context_with(|| format!("error parsing spec: {name}"))?;
	Ok(ops)
}

fn parse_line(line0: &str, add: &mut impl FnMut(Opcode, Line)) -> rootcause::Result<()> {
	let line = line0.split('#').next().unwrap().trim();
	let mut words = line.split_whitespace();
	let Some(first) = words.next() else {
		return Ok(());
	};
	if first == "import" {
		let from = words.next().context("import missing source")?;
		let range = words.next().context("import missing range")?;
		assert!(words.next().is_none());
		let (a, b) = range.split_once("..").context("invalid import range")?;
		let a = a.parse::<Opcode>().context("invalid import range start")?;
		let b = b.parse::<Opcode>().context("invalid import range end")?;
		let include = super::lines_for(from).context_with(|| format!("unknown import source: {from}"))?;
		for (code, line) in include.range(a..b) {
			add(*code, line.clone());
		}
	} else if let Ok(code) = first.parse() {
		let mut line = Line::default();
		for word in words {
			if let Some(name) = word.strip_prefix('\'').and_then(|w| w.strip_suffix('\'')) {
				line.name.push_str(name);
			} else if let Some(alias) = word.strip_prefix("~'").and_then(|w| w.strip_suffix('\'')) {
				line.aliases.push(alias.to_owned());
			} else {
				line.parts.push(word.parse().context_with(|| format!("invalid part: {word}"))?);
			};
		}
		add(code, line);
	} else {
		rootcause::bail!("invalid line start: {first}");
	}
	Ok(())
}

pub fn parse_spec(game: Game, ops: &Lines) -> Spec {
	Spec { game, ops: build_ops(ops), by_name: build_names(ops) }
}

fn build_ops(ops: &Lines) -> [Option<Op>; 256] {
	let mut out = std::array::from_fn(|_| None);
	for (k, line) in ops {
		assert!(!k.is_empty(), "Empty code in spec");
		let mut op = out[k[0] as usize].get_or_insert_with(Op::default);
		for byte in k.iter().skip(1) {
			if op.child_keys.last().is_none_or(|last| last < byte) {
				op.child_keys.push(*byte);
				op.children.push(Op::default());
			}
			op = op.children.last_mut().unwrap();
		}
		op.name = line.name.clone();
		op.parts = line.parts.clone();
	}
	for (i, op) in out.iter_mut().enumerate() {
		if let Some(op) = op {
			fill_name(op, i as u8, "op", false);
		}
	}
	out
}

fn fill_name(op: &mut Op, byte: u8, prefix: &str, parent_has_name: bool) {
	let has_name = !op.name.is_empty();
	if !has_name {
		if parent_has_name {
			op.name = format!("{}_{:02X}", prefix, byte);
		} else {
			op.name = format!("{}{:02X}", prefix, byte);
		}
	}
	for (child_key, child) in op.child_keys.iter().zip(op.children.iter_mut()) {
		fill_name(child, *child_key, &op.name, has_name);
	}
}

fn build_names(inp: &Lines) -> BTreeMap<String, Opcode> {
	let mut all = BTreeSet::new();
	let mut leaves = BTreeSet::new();
	for op in inp.keys() {
		for p in op.prefixes() {
			leaves.remove(&p);
			all.insert(p);
		}
		leaves.insert(*op);
	}

	let mut by_name = BTreeMap::new();
	let mut put = |op: Opcode, name: String| {
		if leaves.contains(&op)
			&& let Some(prev) = by_name.insert(name.clone(), op)
		{
			panic!("Duplicate name in spec: {prev} and {op} are both named {name}");
		}
	};

	for op in all {
		let mut s = String::from("op");
		for b in op {
			write!(s, "{b:02X}").unwrap();
		}
		put(op, s);
	}

	for op in inp.keys() {
		for p in op.prefixes() {
			if let Some(line) = inp.get(&p)
				&& !line.name.is_empty()
			{
				put(*op, derived_name(&line.name, p, *op));
			}
		}
	}

	// Aliases are added afterwards, since a name that is still in use always wins over a former one.
	let mut by_alias = BTreeMap::new();
	for op in inp.keys() {
		if !leaves.contains(op) {
			continue;
		}
		for p in op.prefixes() {
			if let Some(line) = inp.get(&p) {
				for alias in &line.aliases {
					let name = derived_name(alias, p, *op);
					if by_name.contains_key(&name) {
						continue;
					}
					if let Some(prev) = by_alias.insert(name.clone(), *op)
						&& prev != *op
					{
						panic!("Duplicate alias in spec: {prev} and {op} are both aliased {name}");
					}
				}
			}
		}
	}
	by_name.append(&mut by_alias);

	by_name
}

/// The name of `op`, given that its prefix `p` is named `name`.
fn derived_name(name: &str, p: Opcode, op: Opcode) -> String {
	let mut s = name.to_owned();
	if p.len() < op.len() {
		s.push('_');
		for b in &op[p.len()..] {
			write!(s, "{b:02X}").unwrap();
		}
	}
	s
}
