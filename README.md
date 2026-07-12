# strud

A structured diary CLI. Diary entries are Markdown files with YAML
front-matter: free-text prose plus tracked quantitative data. One file per
day, multiple entries appended per day, each entry carrying its own
front-matter.

Stateless — every invocation reads and writes files on disk. The schema lives
in a `strud.toml` beside the entries. See [`SPEC.md`](SPEC.md) for the full
design.

## Status

Early v1. Build from source. Commands: `init`, `new`, `list`.

## Prerequisites

Rust toolchain (cargo), edition 2024. Developed on 1.96.

## Build

```
make            # cargo build
make install    # installs `strud` to ~/.cargo/bin
```

Or directly: `cargo build`, `cargo install --path .`.

## Usage

```
strud init ~/Documents/strud          # scaffold dir + strud.toml + template
strud new                             # prompt metrics, then $EDITOR for the body
strud new --date 2026-07-12T22:40     # backdated entry
strud list                            # last 14 days, table
strud list --since 2026-07-01 --until 2026-07-31
strud list --last 7
strud list --raw                      # dump underlying Markdown
```

The diary directory resolves to the `--dir` flag if given, else `~/.strud/`.

## File format

One `YYYY-MM-DD.md` file per day. Each entry is a `---`-delimited front-matter
block followed by a Markdown body; entries are blank-line separated.

```markdown
---
date: 2026-07-12T08:15
mood: 4
sleep_hours: 7.5
tags: [morning, gym]
---
Quick morning workout.
```

`date` (ISO datetime, minute precision) is required on every entry. Unknown
front-matter keys (e.g. `tags` above) are preserved verbatim but not validated
or given a `list` column.

## Schema (`strud.toml`)

Metrics are declared with a type and optional range. v1 supports `int`,
`float`, `enum`, and `bool`. All metrics are optional at entry time.

```toml
[[metric]]
name = "mood"
type = "int"
min = 1
max = 5

[[metric]]
name = "energy"
type = "enum"
values = ["low", "medium", "high"]

[[metric]]
name = "exercised"
type = "bool"
```

## Body template

`strud new` opens `$EDITOR` (fallback `vi`) pre-filled from
`default.template.md` in the diary directory. Edit that file anytime to change
what new entries start with. `strud init` writes it once and never overwrites
it; delete or edit it to reset.

## Development

```
make help      # list targets
make build     # compile
make test      # run tests
make clippy    # lint (-D warnings)
make fmt       # format
make demo      # scaffold a throwaway diary in ./demo-diary
```