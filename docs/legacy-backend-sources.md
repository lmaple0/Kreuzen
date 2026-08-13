# Legacy backend sources and reproducibility

The legacy adapter uses Git dependencies pinned to the maintained Aureole fork:

- `calmare`
- `themelios`

The dependency is pinned to commit
`a9f2707` (`Limit truncated name tables to Sky FC`). It includes the
ED6/ED7 scenario models, game-specific instruction sets, explicit codecs,
native ED7 layout support, EOF-terminated ED6 names and compilable flat
control-flow output. FC may omit trailing empty character names, while SC and
the 3rd retain their explicit empty-name entries.

This removes the sibling-directory build assumption and makes Cargo resolve the
exact reviewed source revision. The dependency commit must exist on the public
fork before Kreuzen is pushed. No Aureole source is copied into this repository.

The modern backend's pre-existing sibling dependencies are also pinned to their
public repositories:

- `falcom-sjis` at `8bf7d1d151081fd1721a4fee7b73274c54b5bc25`;
- `Gospel` at `9f89bce9d39516d5e7b3c07de510c868af50e134`.

The inspected Aureole checkout still has no root license file. Resolve the
license/provenance question before publishing binary releases; a successful
build is not itself redistribution permission.

Full game scripts and extracted proprietary assets must not be committed.
`tools/New-CorpusManifest.ps1` records relative paths, sizes and SHA-256 hashes
for local regression corpora without copying their contents.
