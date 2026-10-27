# Calibre-Web-Automated for lemonfiber

**Reading and organising your ebook library.**
Calibre-Web-Automated is one of the parts the stack lemonfiber ships is built from.
Without it: No ebook reader.

## How lemonfiber carries it

The default bundle in
[lemonfiber-plugins](https://github.com/lemonfiber/lemonfiber-plugins) pins this
repository by a revision and the digests of its manifest and its tree, and
lemonfiber compiles that tree in. lemonfiber installs Calibre-Web-Automated with
the adapter this repository holds, which speaks lemonfiber's `owner.claim` and
`media.serve` contracts in front of Calibre-Web-Automated: a Rust crate built on
the core's adapter kit, into the image
`ghcr.io/lemonfiber/calibre-web-automated-adapter`, published from every commit
on `main` and from each tag the release train cuts. It takes the administrator a
fresh Calibre-Web-Automated signs in with its published name and password,
renames it and gives it the password lemonfiber mints, reads what books it holds,
and asks it to look over its library again. lemonfiber reaches
Calibre-Web-Automated through this adapter alone. lemonfiber mints the
administrator's name and a password in the shape Calibre-Web-Automated's password
rules accept, and writes them into the adapter's `upstream.json`.

## What is here

| Path | What it is |
| --- | --- |
| `plugin.toml` | The part's manifest |
| `recordings/calibre-web-automated/` | The responses its proofs are held to |
| `src/` | The adapter |
| `conformance/owner.claim@1/` | The adapter's answer to each conformance case, in front of the pinned Calibre-Web-Automated |
| `conformance/media.serve@1/` | The adapter's answer to each conformance case, in front of the pinned Calibre-Web-Automated |
| `Dockerfile` | The adapter's image |
| `targets.toml` | The lemonfiber its checks run against, and the core the adapter is built on |
| `proofs.json` | What those checks said, committed |

## Checks

The plugin verdict is one job of spec's
[`first-party-plugin`](https://github.com/lemonfiber/spec/blob/main/.github/workflows/first-party-plugin.yml)
workflow: the manifest as the lemonfiber `targets.toml` names reads it, every
proof against its recording, and every pinned digest. The adapter is held to its
format, its lints, its tests, full line coverage and its dependency policy, its
image runs as lemonfiber runs it, and `recordings` takes every recording again
against the pinned Calibre-Web-Automated. `just ci` runs the gates that need
nothing but this tree.

## Licence

The code and data in this repository are under the licence in `LICENSE`.
Calibre-Web-Automated itself is under GPL-3.0-only and is not distributed here:
this repository names an image, it does not contain one.
