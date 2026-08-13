# Kreuzen

[简体中文说明](README_CN.md)

A decompiler for *Trails of Cold Steel I-IV*, *Trails into Reverie*, and *Tokyo Xanadu eX+*.
This maintained fork also provides an explicit legacy backend for the PC releases of the
Crossbell games and *Trails in the Sky*.

Verified local corpus coverage currently includes *Trails from Zero*, *Trails to Azure*,
and the PC releases of all three *Trails in the Sky* games. FC was validated against three
separate 491-file corpora from a legally installed Chinese-patched copy. That installation
also contains other mods, so it is not a clean retail baseline. Static roundtrip coverage
does not imply in-game runtime validation; see the reports under [`docs`](docs/).

Kreuzen can roundtrip most modern scripts bytewise, with the remainder being things that are
either inconsequential or broken in the original scripts. Legacy corpus reports separately
record byte differences and second-decompile source stability.

<details><summary>Non-roundtripping scripts</summary>

(Counts are not counting language duplicates)

- 12 scripts where AlgoTable is malformed
- 3 scripts where ActionTable is malformed
- 2 scripts have malformed btlsets
- 1 script in Reverie has an unconventional preload table
- 1 script in CS3 has a broken jump label that would probably crash the game
- One book in English TX, and 14 in Japanese CS3/CS4, have extra pages that are erroneously cut out and are restored

</details>

## Usage

For basic usage, drag a supported script or source file, or a folder containing such, onto the executable. Modern games use `.dat` and `.krz`; the Sky/Crossbell backend uses `._SN`/`.bin` and `.clm`. Outputs will be placed next to the input. For commandline usage, read `--help`.
On Windows, launching the executable without an input displays the full help and waits for a key press instead of immediately closing.
Kruzen will attempt to guess the game based on the containing folder name, but you can override this detection with either `--game cs1` or by renaming the executable itself to `kreuzen-cs1.exe`.

Legacy games require an explicit game profile when the install path is not recognizable. Use
`--legacy-layout native` or `--legacy-layout themelios` for ED7 layouts; Kreuzen does not
silently retry another layout after a parse failure.

### Text encodings and charmaps

Use `--enc sjis`, `--enc utf8`, or `--enc gbk` to override the script text encoding. For CS1 and CS2, files below `dat` and `dat_us` are automatically detected as Shift-JIS and UTF-8 respectively; GBK currently requires the explicit option. Decompiled `.krz` headers retain explicit `sjis` and `gbk` declarations.

`--charmap <file>` applies a reversible custom glyph map during both compilation and decompilation. The file uses one `HEX=GLYPH` entry per line; each right-hand side must be exactly one Unicode character. Empty lines and lines beginning with `#` are ignored. For example:

```text
# The game/font renders CP932 ㈱ as a heart glyph.
878A=♥
```

Byte sequences must be prefix-free. Prefix conflicts, duplicate glyphs, control bytes, empty entries, and base-encoded text that collides with reserved charmap bytes are rejected because they cannot round-trip unambiguously. Kreuzen also handles the Falcom `㈱`/`♥` substitution by default when using Shift-JIS.
