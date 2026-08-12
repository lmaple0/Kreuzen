# Legacy backend sources and reproducibility

The P0 adapter currently uses local path dependencies from the sibling
`Aureole` checkout:

- `calmare`
- `themelios`

The inspected checkout is at commit
`f91ecdba224d61db2ae4e46b1fcddaf98f8c7577`. It already implements ED6 and
ED7 scenario models and the corresponding game-specific instruction sets.

This is deliberately a local integration prototype. Before this branch becomes
the fork's public default branch, replace the sibling-directory assumption with
a reproducible dependency arrangement. Also confirm the upstream license or
obtain permission before copying source into this repository; no root license
file was found in the inspected checkout.

Full game scripts and extracted proprietary assets must not be committed.
`tools/New-CorpusManifest.ps1` records relative paths, sizes and SHA-256 hashes
for local regression corpora without copying their contents.
