# strud — Specification

**strud** is a structured diary CLI. It stores diary entries as Markdown files with YAML front-matter, combining free-text prose with tracked quantitative data. Markdown is a deliberate choice: files stay human-readable, hand-editable, and diff-friendly under git.

- **Language:** Rust
- **Form:** command-line tool, run via `cargo run` / `strud` binary
- **Distribution:** personal / from source (no published package in v1)
- **State:** stateless — every invocation reads and writes files on disk; the tool holds no state between runs

---

## 1. Storage model

### One file per day, entries appended

A diary directory contains one Markdown file per calendar day. A new entry on a day that already has a file is **appended** to that file; a new entry on a day with no file creates the file.

File layout inside the diary directory:

```
<diary-dir>/
  strud.toml            # schema + config (see §3)
  default.template.md   # body template, user-editable (see §6)
  2026-07-12.md
  2026-07-13.md
  ...
```

Files are named `YYYY-MM-DD.md` (zero-padded, Gregorian). They are **flat** in the diary directory — no year/month subdirectories in v1.

### Entry format: one front-matter block per entry

Markdown allows only one top-of-file front-matter block, but a day file holds multiple entries that each need their own structured data. The resolution: **each entry is its own `---`-delimited front-matter block followed by a body**, and entries are concatenated within the day file, separated by a single blank line.

A day file is therefore a **sequence of YAML documents**, each followed by Markdown body text. The parser splits on `---` boundary lines (see §4 for the exact parsing rule).

Example day file `2026-07-12.md`:

```markdown
---
date: 2026-07-12T08:15
mood: 4
sleep_hours: 7.5
tags: [morning, gym]
---
Quick morning workout, felt decent. Need to drink more water today.

---
date: 2026-07-12T22:40
mood: 3
sleep_hours:
tags: [tired]
---
Long day at work. Skipped dinner, regret it.
```

### Reserved front-matter field

- `date` — **required on every entry.** A full ISO 8601 datetime. The date component **must** match the day file it lives in; the time component orders multiple entries within a day.

No other front-matter fields are reserved in v1 (e.g. `tags` above is just an unknown key — see §5).

### Body

The free-text prose following each entry's front-matter, up to the next `---` boundary (or end of file). It is plain Markdown; the tool does not parse or validate it. An empty body is allowed.

---

## 2. Diary directory resolution

The diary directory is resolved simply: the `--dir <path>` flag if given,
otherwise `~/.strud/`.

`strud.toml` lives **at the root of the diary directory** (see §3), so in the
default case config and entries coexist in `~/.strud/`. When `--dir` overrides,
the tool looks for `strud.toml` inside that directory.

The tool errors clearly if the resolved directory does not contain a
`strud.toml` and suggests running `strud init`.

---

## 3. Schema and config (`strud.toml`)

The schema is declared in TOML at `<diary-dir>/strud.toml`. v1 supports `int`, `float`, `enum`, and `bool` value types.

### Metrics

Every metric has a `type`. Number types accept optional `min`/`max` (inclusive) for range validation. `enum` requires a `values` list. **All metrics are optional** — the schema defines types and ranges, not presence. A user may skip any metric at entry time (see §6).

```toml
[[metric]]
name = "mood"
type = "int"
min = 1
max = 5

[[metric]]
name = "sleep_hours"
type = "float"
min = 0
max = 24

[[metric]]
name = "energy"
type = "enum"
values = ["low", "medium", "high"]

[[metric]]
name = "exercised"
type = "bool"
```

Metric names must be valid YAML map keys (lowercase, underscore-separated by convention). Names are case-sensitive. The reserved name `date` may not be reused for a metric.

---

## 4. Parsing rules (file → entries)

A day file is parsed into an ordered list of entries:

1. The file is scanned line-by-line for `---` lines. A `---` that is the first non-blank line of the file (or the first line following a previous entry's body) opens a front-matter block; the matching closing `---` ends it. Everything between is the YAML front-matter; everything after, up to the next opening `---` or EOF, is the body.
2. Front-matter is parsed as YAML into a map. `date` is parsed as a datetime; declared metrics are coerced to their schema type; unknown keys are preserved as-is (see §5).
3. Entries are ordered within the file by their position (which equals their `date` order, since `strud new` inserts in time order — see §6). If a file is hand-edited out of order, `list` should still order by the `date` field, not file position.

### Robustness

- A file with no `---` block at all is treated as a single entry with an empty front-matter (the whole file is body). The tool should warn, since `date` is then missing.
- Malformed YAML in a front-matter block must produce a clear error naming the file and line, not a panic.
- Trailing whitespace / blank lines between entries are ignored.

---

## 5. Validation and unknown keys (lenient)

Validation applies **only to declared metrics** that are actually present in an entry's front-matter:

- **Type:** a present value must coerce to the metric's type. `int`/`float` must be numeric; `bool` must be `true`/`false`; `enum` must be one of `values`.
- **Range:** `int`/`float` values with `min`/`max` must fall within `[min, max]` inclusive.
- **Required `date`:** every entry must have a `date`; its date component must match the day file name.

**Unknown keys are lenient:** front-matter keys not in the schema (e.g. `tags`) are **preserved verbatim** through round-trips and shown in `--raw` dumps, but they are not type-checked and are not given columns in `list`. This keeps the tool usable for ad-hoc metadata without forcing schema edits.

A validation failure (bad type, out of range, missing `date`, or date/file mismatch) during `strud new` **aborts the save** with a clear message; during `strud list` it reports the offending entry but continues listing the rest.

---

## 6. Commands

### `strud init [<dir>]`

Scaffolds a new diary. `<dir>` defaults to the resolution default (§2). Creates the directory if missing and writes two files:

- a starter `strud.toml` containing the example schema from §3 (commented to be editable), and
- a `default.template.md` body scaffold, e.g.:

  ```markdown
  ## Notes

  ## Wins
  ```

  The user edits this file at any time to change what `strud new` opens with.

`--force` allows overwriting an existing `strud.toml` only. `default.template.md` is user content: `init` writes it only if it does not already exist, and never overwrites it. To reset the template, the user deletes or edits the file directly.

### `strud new [--date <datetime>] [--dir <path>]`

Captures a new entry in two phases:

1. **Prompt metrics.** For each declared metric, prompt interactively in the terminal, in schema order. Validate the input against type and range immediately, re-prompting on error (empty input = skip that metric; remember, all metrics are optional). `enum` prompts show the allowed values; `bool` accepts `y/n`.
2. **Edit body.** Open `$EDITOR` (fallback `vi`) on a temp file **pre-filled from the body template** at `<diary-dir>/default.template.md`. The template is plain Markdown for the body only (the tool builds the front-matter itself from the prompted metrics, so the template must not include front-matter). If the template file is missing, the editor opens empty. On editor exit, read the body (empty allowed).

The entry's `date` defaults to **now** (local time) as an ISO datetime with minute precision (`YYYY-MM-DDTHH:MM`). `--date` overrides to any ISO datetime (past or future); its date component selects which day file the entry lands in.

**Append and order:** locate (or create) the day file for the entry's date. Insert the new entry at the position that keeps the file sorted by `date` (ascending). If the file is empty, it becomes the first entry. The entry is written as a `---` front-matter block + body, blank-line-separated from neighbors. Only declared metrics that the user filled are written; skipped metrics are omitted entirely (not written as null).

After writing, print a one-line confirmation with the file path and the entry's `date`.

### `strud list [--date <date>] [--since <date>] [--until <date>] [--last <N>] [--dir <path>] [--raw]

Reads all day files in the diary directory, parses entries, and prints them.

- **Default (table):** one row per entry. Columns are, in order: `date` (ISO datetime), then one column per declared metric (in schema order). Skipped/absent values render as a blank cell. Filters:
  - `--date YYYY-MM-DD` — only that calendar day.
  - `--since YYYY-MM-DD` / `--until YYYY-MM-DD` — inclusive date range bounds.
  - `--last N` — the last N entries by `date` (default when no other filter is given: the last 14 days of entries).
  - Filters compose; `--last` applies after date filtering.
- **`--raw`:** instead of the table, concatenate the underlying Markdown of each matching day file to stdout (the file content, in date order), preserving front-matter and body exactly.

Exit non-zero if no entries match (with a message), so scripts can detect empty results.

---

## 7. Non-goals for v1

- No `stats`/aggregation, `edit`, `show`, or `search` subcommands (deferred; design leaves room for them).
- No charts or export beyond `--raw`.
- No encryption or sync.
- No year/month subdirectories.
- No cross-day validation or streaks.
- No published binaries or crates.io release — build from source.

---

## 8. Suggested dependencies & project structure

Suggested crates (the implementer may choose equivalents):
- `clap` (derive) — CLI parsing.
- `serde` / `serde_yaml` — front-matter and schema types. (Note: serde_yaml is maintained but the ecosystem also offers alternatives; pick one that round-trips unknown keys.)
- `toml` — read `strud.toml`.
- `chrono` — datetimes and date arithmetic; default to local time.
- `anyhow` — error handling with context; `thiserror` for typed parse/validation errors if preferred.
- `dialoguer` or `inquire` — interactive metric prompting.
- `tempfile` + `edit` (or a small `$EDITOR` wrapper) — body editing.

Suggested module layout:

```
src/
  main.rs          # clap arg parsing, dispatch
  config.rs        # load/validate strud.toml, Metric struct
  diary.rs         # diary dir resolution (§2)
  entry.rs         # Entry struct, front-matter + body model
  parse.rs         # day-file <-> Vec<Entry> (§4)
  validate.rs      # type/range/date validation (§5)
  commands/
    init.rs
    new.rs
    list.rs
```

---

## 9. Worked example session

```
$ strud init ~/Documents/strud
Created diary at ~/Documents/strud with starter strud.toml.

$ strud new
mood (int 1-5) [blank to skip]: 4
sleep_hours (float 0-24) [blank to skip]: 7.5
energy (low|medium|high) [blank to skip]: medium
exercised (y/n) [blank to skip]: y
  # opening editor for body...
Added entry 2026-07-12T09:10 to ~/Documents/strud/2026-07-12.md

$ strud new --date 2026-07-12T22:40
... (prompts) ...
Added entry 2026-07-12T22:40 to ~/Documents/strud/2026-07-12.md

$ strud list --date 2026-07-12
date                 mood  sleep_hours  energy  exercised
2026-07-12T09:10     4     7.5          medium  true
2026-07-12T22:40     3                  low     false
```

The second entry was saved with front-matter `mood: 3`, `energy: low`, `exercised: false`, plus an unknown key `tags: [tired]`. The `tags` key is preserved in the file and shown by `strud list --raw`, but it has no column in the table view (see §5).