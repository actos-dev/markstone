# markstone — Plan

> Status: draft. This file records the decisions made **before** any
> implementation. Whoever implements it works from here; if the work needs to
> deviate, this file is updated first, then the code is written.

---

## 1. What this package is and why it exists

markstone is a single core that turns markdown into safe HTML, plus thin
binding layers that carry that core into eight languages.

The origin is Actos: bodies on the platform are shared as markdown, and today
the server computes a `body_html` field and ships it inside the JSON next to
the raw text. That is going away. Only raw text will cross the wire, and every
client will produce HTML locally.

There are three reasons for this:

1. **The trust boundary shrinks.** Trusting server-rendered HTML means
   trusting that everything arriving over the wire really is HTML. A bug on
   the server, or a mirror sitting in the middle, can inject HTML into every
   client. Sending raw text and rendering it with an audited local library
   closes that surface.
2. **Not every interface uses HTML.** The TUI in the CLI cannot paint HTML at
   all and currently leaves that field empty. Desktop and mobile interfaces
   want a native component tree too.
3. **Live preview while writing should not need a server round trip.**

### Two function families

The package splits in two, and the split is structural rather than a naming
convention:

- **Generic family.** Pure CommonMark and GFM. Knows nothing about Actos.
  Anyone can use it.
- **Actos family.** Runs the generic pipeline and then adds Actos-specific
  passes on top, such as `@mention` and `#tag` linking. Separate crate,
  separate function names, separate tests.

Staying generic is not a goal in itself. The goal is that Actos clients get
everything they actually need from a single install. Having eight clients each
write their own mention-linking pass would be exactly the duplication we are
trying to avoid. As long as the split stays at the function level, the generic
side stays clean.

### What we promise

- The same input produces **byte-for-byte identical** output in every
  language.
- The HTML output is safe to paint directly into a browser, whatever the
  source text is. This holds for both function families.
- Installation is a single step. Users never see a C compiler, a Rust
  toolchain, or a build step.

### What we do not promise

- Syntax highlighting. Out of scope, code blocks come out as plain
  `<pre><code>` carrying the language as a class.
- Configuration. **No function takes options.** Every option is a place where
  one language's default can drift, which breaks the "same output everywhere"
  promise directly. When a behavioral difference is needed we add a function,
  not a flag. The Actos family is exactly how that rule plays out.
- Markdown generation. One direction only: markdown in, HTML or AST out.

---

## 2. Architecture

```
        ┌────────────────────────┐   ┌──────────────────────────┐
        │  core (unsafe denied)  │◄──┤  actos (unsafe denied)   │
        │  parse → AST →         │   │  mention / tag pass      │
        │  sanitize → HTML/JSON  │   │                          │
        └───────────┬────────────┘   └────────────┬─────────────┘
                    └──────────────┬──────────────┘
                    ┌──────────────┴──────────────┐
                    │  abi (the only unsafe)      │
                    │  cdylib + staticlib, C ABI  │
                    └──────────────┬──────────────┘
                                   │
   ┌──────┬──────┬───────┬─────────┼───────┬───────┬──────────┐
   │ rust │  py  │ node  │  wasm   │  go   │  jvm  │  dotnet  │
   └──────┴──────┴───────┴─────────┴───────┴───────┴──────────┘
```

The Rust binding does not go through FFI, it uses the crates directly.
Everything else calls the same `abi` surface.

### 2.1 Parser: comrak

**Decision: use comrak (0.55 series). Do not write a parser from scratch.**

Rationale:

- comrak is the Rust port of cmark-gfm. Its CommonMark 0.31 conformance and
  its GFM extensions have been exercised for years.
- It produces a real node tree. One of our two outputs is the AST, so that
  comes for free instead of us rebuilding a tree from an event stream.
- It escapes raw HTML by default, so we do not need the "filter raw HTML
  events by hand" trick.

Options considered and rejected:

- **pulldown-cmark:** faster, and already a dependency in the Actos backend,
  but it hands back an event stream and has no flag that disables raw HTML. We
  would have to build the AST ourselves.
- **Writing our own:** the spec suite has over six hundred examples, and the
  edge cases around emphasis delimiters and lazy list continuation eat most of
  the work. In return nothing changes for the user, because the output has to
  match CommonMark anyway. The value of this project is not in the parser, it
  is in the interface and the distribution chain that give eight languages one
  identical output.

### 2.2 Supported syntax

The baseline is **CommonMark 0.31 plus all of GFM**: tables, strikethrough,
autolinks, task lists, and footnotes. We do not cherry-pick GFM features.
"CommonMark + GFM" is one named, testable contract; a trimmed subset would be
a list that needs a fresh justification every time.

### 2.3 Sanitization: at the AST level, no ammonia

**Decision: sanitize on the AST rather than by passing output through a
separate sanitizer.**

The conventional approach would be to run comrak's output through ammonia. We
are not doing that:

- ammonia pulls in html5ever. In the WASM binary built for browsers that
  weight lands directly in the download size.
- A sanitizer exists to clean HTML of unknown provenance. The provenance here
  is our own renderer over a closed set of node types. There is no path for a
  tag we did not choose to reach the output.
- Using a different sanitization path on native and on WASM would mean the
  output can diverge, which breaks the one promise this project makes.

The untrusted inputs that reach the output are few, and all of them are
handled in the tree:

| Input | Rule |
|---|---|
| Text nodes | `&`, `<`, `>`, `"` are escaped |
| Link and image URLs | Only `http`, `https`, `mailto`. Anything else is dropped and the node degrades to plain text |
| Raw HTML blocks and inline raw HTML | Dropped in the tree, never reach the output |
| Code block language tag | Anything outside `[A-Za-z0-9_+-]` is stripped, written as a class with a `language-` prefix |
| Heading `id` generation | Not done |
| Invisible and bidi control characters | Stripped from the input before parsing |

That last row matters: `U+200B`..`U+200D`, `U+FEFF`, `U+2060`, `U+00AD`,
`U+202A`..`U+202E`, `U+2066`..`U+2069`. These are used to reverse the visual
order of text and to impersonate visually, so stripping them belongs in the
render layer.

External links get `rel="nofollow noopener noreferrer"`. The Actos backend
already does this today and it is the safe default.

**What we do not do:** Unicode NFC normalization. That is a storage and
comparison decision, not a rendering one. A markdown renderer should not
silently rewrite user text. Actos already applies it on write.

**We keep the second line of defense in the test suite instead.** Every golden
HTML output is run through ammonia in CI and must come back **unchanged**.
That buys the assurance of a second layer without the runtime cost or the
binary size. If that test breaks, something we did not expect is in our
output.

### 2.4 Limits

Fixed, not configurable, documented:

| Limit | Value | On breach |
|---|---|---|
| Input size | 4 MiB | `INPUT_TOO_LARGE` |
| Block nesting depth | 64 | `DEPTH_EXCEEDED` |

The depth limit is not cosmetic. The AST output is a nested tree, so the
consumer in every language walks it recursively. Without a limit, anyone
sending five hundred levels of nested block quotes could blow the stack in
every client. When the limit is exceeded we return an error rather than
silently truncating.

The core's own tree walks are written **without recursion**. Even on inputs
that sit just under the limit the core must not consume stack.

---

## 3. Actos extensions

Separate crate (`actos/`), separate ABI functions, separate tests. It runs the
generic pipeline and then makes one more pass over the tree.

### 3.1 Mentions and tags

| Syntax | AST node | HTML |
|---|---|---|
| `@username` | `mention` | `<a href="/u/username" class="mention">@username</a>` |
| `#tag` | `tag` | `<a href="/t/tag" class="tag">#tag</a>` |

URLs are relative. Every Actos web surface is same-origin, so absolute URLs
are unnecessary, and keeping them relative avoids having to add a "base URL"
option to the function. The paths match the existing frontend routes exactly:
`app/u/[username]` and `app/t/[name]`.

### 3.2 Matching rules

All of these are pinned by tests:

- **Text nodes only.** Code spans, code blocks, autolinks, and link
  destinations are left untouched.
- **No matching inside link text.** `[@foo](https://…)` must not produce a
  nested link.
- **Username is `^[a-z0-9_]{3,32}$`, tag is `^[a-z0-9][a-z0-9-]{0,31}$`.** A
  candidate that does not match stays as plain text. That is why `@Foo` does
  not match: usernames in the Actos schema are lowercase already.
- **The preceding character cannot be a letter, digit, or underscore.**
  Without this, `mail@example.com` would be read as a mention.
- **Trailing punctuation stays outside.** In `@foo.` the name is `foo` and the
  period continues as text. The permitted character set gives this for free.

The two regular expressions here mirror the `ck_actors_username_format` and
`ck_tags_name_format` constraints in the Actos migrations. If one changes and
the other is forgotten, a valid mention stops linking or we emit a link to a
profile that does not exist. That correspondence is pinned by an explicit test
under `conformance/` and noted in both files.

### 3.3 Consumers that do not know these types

`mention` and `tag` nodes carry a `text` field alongside `username` or `name`,
holding the source form (`@foo`). A consumer that does not recognize these
types can then at least display the node as plain text.

---

## 4. The C ABI contract

`abi/include/markstone.h`:

```c
typedef enum {
  MARKSTONE_OK                 = 0,
  MARKSTONE_ERR_NULL_ARGUMENT  = 1,
  MARKSTONE_ERR_INVALID_UTF8   = 2,
  MARKSTONE_ERR_INPUT_TOO_LARGE= 3,
  MARKSTONE_ERR_DEPTH_EXCEEDED = 4,
  MARKSTONE_ERR_INTERNAL       = 5
} markstone_status;

/* --- Generic family: pure CommonMark + GFM --- */

/* The input need not be NUL-terminated; the length is passed explicitly.
   On success *out points at a NUL-terminated UTF-8 buffer and *out_len is
   its byte length excluding the NUL. On error *out and *out_len are left
   untouched. The returned buffer is released with markstone_free. */
markstone_status markstone_to_html(const char *input, size_t input_len,
                                   char **out, size_t *out_len);

markstone_status markstone_to_ast(const char *input, size_t input_len,
                                  char **out, size_t *out_len);

/* --- Actos family: the generic pipeline plus the mention/tag pass --- */

markstone_status markstone_actos_to_html(const char *input, size_t input_len,
                                         char **out, size_t *out_len);

markstone_status markstone_actos_to_ast(const char *input, size_t input_len,
                                        char **out, size_t *out_len);

/* --- Shared --- */

/* Releases a buffer returned by markstone_to_*. Does nothing if ptr is NULL.
   len must be exactly the out_len that call handed back. */
void markstone_free(char *ptr, size_t len);

/* Static semver string, never freed. */
const char *markstone_version(void);

/* AST JSON schema version. Increments when the schema breaks. */
uint32_t markstone_ast_schema_version(void);
```

Why the contract is shaped this way:

- **The input length is explicit.** Strings in Go, Java, and C# are not
  NUL-terminated; we are not pushing a copy-and-terminate cost onto every
  binding. An embedded NUL in the input is not an error, it is part of the
  text.
- **The output is both NUL-terminated and length-carrying.** C callers keep
  their habits, but since the length is handed back no binding calls `strlen`,
  and the same number is what `markstone_free` needs.
- **One release function.** All four output functions share one memory
  contract. A binding author never has to work out which output frees how.
- **Status in the return value, data in an out parameter.** Empty output and
  an error can never be confused.
- **The family split lives in the names.** There is no flag parameter. A flag
  would be the first place a binding's default could drift.

**Panic boundary.** Every ABI function body is wrapped in `catch_unwind`. A
Rust panic crossing an FFI boundary is undefined behavior and takes the host
application down. A caught panic becomes `MARKSTONE_ERR_INTERNAL`. The goal is
that the core never panics; this is only the last line of defense.

**unsafe policy.** The `core` and `actos` crates open with
`#![forbid(unsafe_code)]`. `unsafe` appears only in the `abi` crate, for
pointer conversion and buffer handoff, and every block carries its
justification above it.

**Thread safety.** The functions are stateless with no shared mutable state,
so it does not matter how many threads call them at once. This goes into the
contract and is covered by a test.

---

## 5. The AST JSON schema

The type is necessarily a string. The only common denominator across eight
languages is JSON. Binary formats add another dependency to every binding and
break the simple-install requirement, and a bespoke text format would force us
to write a parser in each language.

The shape is a **nested tree**, not a flat event list. Serializing an event
list would be easier, but then the consumer has to build its own tree in every
language, which puts the work on the wrong side.

```json
{
  "schema": 1,
  "root": {
    "type": "document",
    "pos": [1, 1, 4, 12],
    "children": [
      {
        "type": "heading",
        "level": 2,
        "pos": [1, 1, 1, 9],
        "children": [{ "type": "text", "value": "Heading", "pos": [1, 4, 1, 9] }]
      }
    ]
  }
}
```

Node types and their type-specific fields:

| Type | Fields | Takes children | Family |
|---|---|---|---|
| `document` | — | yes | both |
| `paragraph` | — | yes | both |
| `heading` | `level` 1..6 | yes | both |
| `block_quote` | — | yes | both |
| `list` | `ordered`, `start`, `tight` | yes | both |
| `list_item` | — | yes | both |
| `task_item` | `checked` | yes | both |
| `code_block` | `language`, `value` | no | both |
| `thematic_break` | — | no | both |
| `table` | `alignments` | yes | both |
| `table_row` | `header` | yes | both |
| `table_cell` | — | yes | both |
| `text` | `value` | no | both |
| `emphasis` | — | yes | both |
| `strong` | — | yes | both |
| `strikethrough` | — | yes | both |
| `code` | `value` | no | both |
| `link` | `url`, `title` | yes | both |
| `image` | `url`, `title` | yes | both |
| `soft_break` | — | no | both |
| `line_break` | — | no | both |
| `footnote_definition` | `name` | yes | both |
| `footnote_reference` | `name` | no | both |
| `mention` | `username`, `text` | no | Actos only |
| `tag` | `name`, `text` | no | Actos only |

Rules:

- **`pos` is on every node:** `[start line, start column, end line, end
  column]`, 1-based. Live preview needs it to know which node the cursor sits
  in, and adding it later would break the schema.
- **Text values are unescaped.** The JSON carries the raw text. HTML escaping
  is the renderer's job; an AST consumer may not be producing HTML at all.
- **URLs are filtered in the tree too.** A link with a disallowed scheme
  appears in the AST as plain text, not as a `link` node. All four output
  functions make the same security decisions; the AST path is not an escape
  hatch.
- **There is no raw-HTML node type.** Because it is dropped from the tree.
- **Unknown fields are ignored.** If the schema grows, `schema` increments;
  consumers should read it and fail on an incompatible version.

---

## 6. Languages, packaging, and the reasoning behind each

Seven packages, eight languages. Kotlin and Java share the JVM artifact. Every
package exposes both function families; the split lives at the namespace
level, for example `markstone.to_html` and `markstone.actos.to_html` in
Python.

| Target | Method | Registry |
|---|---|---|
| Rust | Uses the crates directly, no FFI | crates.io |
| Python | PyO3, `abi3` wheels, maturin + cibuildwheel | PyPI |
| Node.js | napi-rs, prebuilt binary per platform | npm |
| Browser | wasm-bindgen, inside the same npm package | npm |
| Go | purego | Go module |
| Java / Kotlin | FFM (Panama), JDK 22+ | Maven Central |
| C# | P/Invoke, `runtimes/{rid}/native/` | NuGet |

Notes and rationale:

- **Node and the browser are one npm package,** split by conditional entry
  points in `exports`. Node gets the native addon, the browser gets WASM.
  Both are compiled from the same Rust source so the output cannot diverge,
  and that is verified by conformance rather than assumed.
- **WASM is mandatory in the browser.** Native code does not run there and
  napi-rs only applies to Node. Apart from this single exception WASM does not
  leak anywhere.
- **On the JVM we chose FFM, not JNI.** FFM is stable in 22; the Actos Kotlin
  SDK targets JVM 17 today and that version will be raised. JNI would work on
  17 but the amount of code to write and maintain grows noticeably.
- **abi3 on Python,** so we do not build a wheel per Python version. The
  release matrix collapses to the platform count instead of multiplying by
  version count.

### Go: purego

**Decision: purego.** No cgo required, works with `CGO_ENABLED=0`, does not
break cross-compilation. Fully static dependency-free builds are very common
in the Go world, and a library that forces cgo is treated as uninstallable in
practice.

The cost is that the shared library has to exist as a file on disk. The
approach:

- The platform-appropriate library is embedded into the module with `embed`.
- On first use it is extracted under `os.UserCacheDir()` into a file named by
  version and content hash. The same version is never extracted twice.
- `/tmp` is not used; it is mounted `noexec` in many environments.
- If the cache directory is `noexec` too, extraction fails. For that case the
  `MARKSTONE_LIBRARY_PATH` environment variable points at a system-installed
  library, and the error message says so directly.

Rejected alternative, **cgo plus a static archive**: the most robust at
runtime, but it demands a C toolchain and makes `CGO_ENABLED=0` builds
impossible.

### License and version policy

- **MIT.** Aligned with `ccharts`.
- **All packages share one version number.** The conformance contract is
  defined per version, so a user must be able to say "markstone 1.4
  everywhere". The cost is that a binding fix in one language means publishing
  a release in the other six with no content change. That cost is accepted; in
  exchange the conformance matrix stays one-dimensional.

---

## 7. Platform matrix

Prebuilt binaries produced on every release:

```
linux   x86_64  gnu
linux   x86_64  musl
linux   aarch64 gnu
linux   aarch64 musl
macos   x86_64
macos   aarch64
windows x86_64
windows aarch64
wasm32  (browser)
```

This matrix is the most expensive and most fragile part of the project.
Writing the code is not what eats time; keeping nine targets green on every
release is. The plan is built around that: the CI matrix comes up as a
skeleton in Phase 1 and each phase adds its binding to it. It is not assembled
in one go at the end.

---

## 8. Repository layout

```
markstone/
├── PLAN.md
├── README.md
├── LICENSE
├── Cargo.toml                 # workspace
├── core/                      # markstone-core, unsafe denied
│   ├── src/
│   └── tests/
├── actos/                     # markstone-actos, unsafe denied
│   ├── src/
│   └── tests/
├── abi/                       # markstone-abi, cdylib + staticlib
│   ├── include/markstone.h
│   └── src/
├── fuzz/                      # cargo-fuzz targets
├── bindings/
│   ├── rust/                  # thin idiomatic wrapper over the crates
│   ├── python/
│   ├── node/                  # napi-rs
│   ├── wasm/                  # wasm-bindgen, folded into the npm package
│   ├── go/
│   ├── jvm/                   # java + kotlin, one artifact
│   └── dotnet/
├── conformance/
│   ├── cases/                 # input + expected HTML + expected AST
│   ├── spec/                  # CommonMark 0.31 examples
│   └── runner/                # language-agnostic comparator
├── scripts/
└── .github/workflows/
```

---

## 9. Test strategy

Four layers, each measuring something different:

1. **Spec conformance (core only).** The CommonMark 0.31 example suite plus
   the GFM extension tests. Examples that do not pass are listed explicitly
   with a reason; none are skipped silently.
2. **Security corpus.** Our own set of XSS and injection inputs: links with
   `javascript:`, `data:`, and `vbscript:` schemes, raw HTML with event
   attributes, `<script>`, `<iframe>`, bidi control characters, quote escapes
   squeezed into a code block language tag. Each one is exercised against all
   four output functions. The Actos family additionally has to show that
   mention and tag links cannot themselves carry an injection.
3. **The ammonia oracle.** Every golden HTML output is run through ammonia in
   CI and must come back unchanged. If that test breaks, something we did not
   expect is in our output.
4. **Cross-language conformance.** Every binding must produce **byte-for-byte**
   the same output as the golden files for the same input set. The input set
   covers both families.

One note on that fourth layer: because every binding calls the same core,
these tests actually measure the bindings rather than the parser. So the suite
is cheaper than it first looks. The focus should be:

- UTF-8 encoding and the host language's string representation, especially
  UTF-16 in Java and C# and byte slices in Go
- Empty input, whitespace-only input, embedded NUL
- The correct error code surfacing when a limit is exceeded
- Memory: each binding calls in a loop and is verified not to leak. The ABI
  layer additionally runs under ASAN and valgrind.

Plus **fuzzing**: continuous random input into the core via `cargo-fuzz`.
Three targets, that it never panics, that the limits actually hold, and that
the output is always valid UTF-8.

---

## 10. Phases

Every phase must be independently mergeable and testable.

**Phase 0 — Skeleton.** Workspace, license, README, an empty CI matrix,
version policy. Output: `cargo test` green on an empty workspace.

**Phase 1 — Core, HTML path.** comrak integration, the sanitization pass over
the tree, a non-recursive HTML renderer, the limits. The spec suite and the
security corpus pass. Output: `markstone-core` stands on its own.

**Phase 2 — Core, AST path.** JSON schema, serialization, schema version. A
test proves both paths make the same security decisions.

**Phase 3 — Actos extensions.** The `markstone-actos` crate, the mention and
tag pass, tests for every rule in section 3, and the test that pins the
regular expressions against the Actos migration constraints.

**Phase 4 — ABI.** The C header, the four functions, the memory contract,
`catch_unwind`, thread tests, fuzz targets, an ASAN run. Output: a `cdylib`
callable from C.

**Phase 5 — Conformance harness.** Golden file format, generator script,
language-agnostic comparator. This has to land before Phase 6; the machine
that validates the first binding must already exist.

**Phase 6 — Rust and Python.** The two easiest targets. They also validate the
conformance harness itself. Output: a publish rehearsal to crates.io and PyPI.

**Phase 7 — Node and the browser.** napi-rs and wasm-bindgen, one npm package,
conditional `exports`. Conformance proves both paths produce the same output.

**Phase 8 — JVM.** FFM, JDK 22, the Java API, and a thin idiomatic layer on
top for Kotlin. One artifact.

**Phase 9 — .NET.** P/Invoke, `runtimes/{rid}/native/` packaging.

**Phase 10 — Go.** purego, the embedded library and cache-directory
extraction, the `MARKSTONE_LIBRARY_PATH` escape hatch.

**Phase 11 — Release.** Automated publication to six registries from a version
tag, prebuilt binaries attached to the GitHub Release, version consistency
checks.

**Phase 12 — Actos integration.** Section 11.

---

## 11. Work on the Actos side

This phase runs in the Actos repositories once markstone ships. There is **no**
backward compatibility; the field is not deprecated, it is removed outright.

### Backend (`actos-backend`)

Rendering has a single call site, so the cleanup is narrow:

- `crates/actos-core/src/text.rs`: delete `render_markdown`, `sanitize_html`,
  `allowed_tags`, `allowed_tag_attributes`, `allowed_url_schemes`, and their
  tests. `normalize_text` and the `validate_*` functions **stay**; the server
  cannot stop validating, and the client is never trusted.
- `Cargo.toml`: drop the `pulldown-cmark` and `ammonia` dependencies.
- `crates/actos-types/src/content.rs`: delete the
  `ContentSummary::body_html` field.
- `crates/actos-api/src/fields.rs`: delete `wants_body_html`.
- `crates/actos-api/src/routes/posts.rs`: delete
  `content_summary_with_body_html`, `content_summary_with_optional_body_html`,
  and the render call gated on `body_format`.
- `routes/feed.rs`, `routes/interactions.rs`, `routes/search.rs`: delete the
  `include_body_html` plumbing.
- `routes/comments.rs`, `routes/notifications.rs`: delete the `body_html`
  query flag.
- Remove `?fields=body_html` selector support and its tests.
- Remove the render assertions in the API tests.
- Add a comment to the `ck_actors_username_format` and `ck_tags_name_format`
  migrations noting that they must stay in sync with the regular expressions
  in markstone's Actos crate.

### SDKs

Across the eight repositories, clean up the model field, the generated schema,
and the tests. Generated schema files are refreshed by regenerating them from
the backend schema, not by hand.

### Frontend

Server components keep rendering, but on the Next.js Node side rather than in
the API. That means the native path of the npm package runs there. The body
still appears in the page source, so search engines and link previews are
unaffected. Client-side live preview uses the WASM path of the same package.
Both places call the Actos family, not the generic one.

### CLI

The TUI uses the Actos AST path, not the HTML path. It builds ratatui spans
from the AST, painting `mention` and `tag` nodes in their own colors. The body
field that is left empty today finally gets filled.
