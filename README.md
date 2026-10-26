# Calibre-Web-Automated for lemonfiber

**Reading and organising your ebook library.**
Calibre-Web-Automated is one of the parts the stack lemonfiber ships is built from.
Without it: No ebook reader.

## How lemonfiber carries it

The default bundle in
[lemonfiber-plugins](https://github.com/lemonfiber/lemonfiber-plugins) pins this
repository by a revision and the digests of its manifest and its tree, and
lemonfiber compiles that tree in.
This repository holds the adapter that speaks lemonfiber's `owner.claim` and
`media.serve` contracts in front of Calibre-Web-Automated: a Rust crate built on
the core's adapter kit, into the image
`ghcr.io/lemonfiber/calibre-web-automated-adapter`, published from every commit
on `main` and from each tag the release train cuts. It takes the administrator a
fresh Calibre-Web-Automated signs in with its published name and password,
renames it and gives it the password lemonfiber mints, reads what books it holds,
and asks it to look over its library again.
`plugin.toml` does not name the adapter yet, so lemonfiber assembles
Calibre-Web-Automated into the stack it ships as a stack service.

## What is here

| Path | What it is |
| --- | --- |
| `plugin.toml` | The part's manifest |
| `recordings/calibre-web-automated/` | The responses its claims are proved against |
| `src/` | The adapter |
| `Dockerfile` | The adapter's image |
| `targets.toml` | The lemonfiber its checks run against, and the core the adapter is built on |
| `proofs.json` | What those checks said, committed |

## Checks

The plugin verdict is one job of spec's
[`first-party-plugin`](https://github.com/lemonfiber/spec/blob/main/.github/workflows/first-party-plugin.yml)
workflow: the manifest as the lemonfiber `targets.toml` names reads it, every
claim against its recording, and every pinned digest. The adapter is held to its
format, its lints, its tests, full line coverage and its dependency policy, and
its image runs as lemonfiber runs it. `just ci` runs the gates that need nothing
but this tree.

## Licence

The code and data in this repository are under the licence in `LICENSE`.
Calibre-Web-Automated itself is under GPL-3.0-only and is not distributed here:
this repository names an image, it does not contain one.
