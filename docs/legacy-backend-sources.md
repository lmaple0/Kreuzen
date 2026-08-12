# Legacy backend sources and reproducibility

The legacy adapter uses Git dependencies pinned to the maintained Aureole fork:

- `calmare`
- `themelios`

The dependency is pinned to commit
`9563e4e` (`Complete Sky ED6 script roundtrip support`). It implements the
ED6/ED7 scenario models, game-specific instruction sets, explicit codecs,
native ED7 layout support, EOF-terminated ED6 names and compilable flat
control-flow output.

This removes the sibling-directory build assumption and makes Cargo resolve the
exact reviewed source revision. The dependency commit must exist on the public
fork before Kreuzen is pushed. No Aureole source is copied into this repository.

The inspected Aureole checkout still has no root license file. Resolve the
license/provenance question before publishing binary releases; a successful
build is not itself redistribution permission.

Full game scripts and extracted proprietary assets must not be committed.
`tools/New-CorpusManifest.ps1` records relative paths, sizes and SHA-256 hashes
for local regression corpora without copying their contents.
