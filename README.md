# markstone

Fast, safe Markdown-to-HTML and AST engine with Actos extensions and bindings across eight languages.

---

## 1. Overview and Motivation

**markstone** is a single core engine that compiles Markdown into safe HTML and structured JSON AST, paired with thin idiomatic bindings across eight programming languages.

Originally built for the **Actos** platform, markstone replaces server-side HTML rendering with verified, audited client-side rendering.

### Why Client-Side Rendering?

1. **Shrinking the Trust Boundary**: Transmitting raw Markdown instead of server-rendered HTML eliminates HTML injection risks from compromised mirrors or server bugs. Clients parse and sanitize content locally using an audited engine.
2. **Native Component Trees**: Non-browser interfaces (such as the Actos terminal UI built with Ratatui, desktop apps, and mobile clients) need structured node trees (AST) rather than HTML strings.
3. **Instant Live Preview**: Interactive authoring surfaces render live previews without round-trips to the server.

### Core Promises

- **Byte-for-byte identical output** across every supported language and platform.
- **Always safe HTML** ready to paint directly into a browser, regardless of input.
- **Zero-toolchain installation** for end users: prebuilt binaries and native packages require no C compiler or Rust toolchain.
- **Zero configuration options**: Functions take no option flags, preventing configuration drift across language ecosystems.

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
                    │  abi (isolated unsafe)      │
                    │  cdylib + staticlib, C ABI  │
                    └──────────────┬──────────────┘
                                   │
   ┌──────┬──────┬───────┬─────────┼───────┬───────┬──────────┐
   │ rust │  py  │ node  │  wasm   │  go   │  jvm  │  dotnet  │
   └──────┴──────┴───────┴─────────┴───────┴───────┴──────────┘
```

The codebase is organized into modular crates:

- **`markstone-core` (`core/`)**: Pure CommonMark 0.31 + GitHub Flavored Markdown (GFM). Handles parsing (via `comrak`), AST-level sanitization, and non-recursive HTML/AST rendering. Strictly `#![forbid(unsafe_code)]`.
- **`markstone-actos` (`actos/`)**: Actos-specific extensions (user `@mention` and `#tag` linking). Strictly `#![forbid(unsafe_code)]`.
- **`markstone-abi` (`abi/`)**: Exposes the stable C ABI via `cdylib` and `staticlib`. Wraps calls in `catch_unwind` panic barriers. The only crate where `unsafe` is permitted for FFI pointer management.

---

## 3. Two Function Families

1. **Generic Family (`markstone_*`)**: Pure CommonMark 0.31 + GFM (tables, strikethrough, autolinks, task lists, footnotes). Completely agnostic of Actos.
2. **Actos Family (`markstone_actos_*`)**: Executes the generic pipeline and applies Actos domain passes:
   - `@username` → `<a href="/u/username" class="mention">@username</a>` (matches `^[a-z0-9_]{3,32}$`)
   - `#tag` → `<a href="/t/tag" class="tag">#tag</a>` (matches `^[a-z0-9][a-z0-9-]{0,31}$`)

---

## 4. Security and Limits

- **AST-Level Sanitization**: Rather than piping output through heavy HTML sanitizers like `ammonia` at runtime (which inflates WASM bundles via `html5ever`), sanitization occurs directly on the AST node tree before rendering.
- **Allowed Schemes**: Only `http`, `https`, and `mailto`. Disallowed schemes degrade links into plain text.
- **Dropped Elements**: Raw HTML blocks and inline raw HTML are stripped in the tree and never reach output.
- **Sanitized Attributes**: Code block language tags are filtered to `[A-Za-z0-9_+-]`. Invisible and bidirectional control characters (`U+200B`..`U+200D`, `U+FEFF`, `U+202A`..`U+202E`, etc.) are stripped.
- **The Ammonia Oracle**: In CI, all golden test HTML outputs are validated against `ammonia`. Any divergence fails the build.
- **Fixed Limits**:
  - Maximum input size: **4 MiB** (`MARKSTONE_ERR_INPUT_TOO_LARGE`).
  - Maximum block nesting depth: **64** (`MARKSTONE_ERR_DEPTH_EXCEEDED`).
  - Non-recursive tree traversal prevents stack overflows on deeply nested documents.

---

## 5. Target Platforms and Prebuilt Binaries

Prebuilt binaries are generated for nine release targets:

| Operating System | Architecture | Toolchain / Target |
|---|---|---|
| Linux | x86_64 | GNU (`x86_64-unknown-linux-gnu`) |
| Linux | x86_64 | musl (`x86_64-unknown-linux-musl`) |
| Linux | aarch64 | GNU (`aarch64-unknown-linux-gnu`) |
| Linux | aarch64 | musl (`aarch64-unknown-linux-musl`) |
| macOS | x86_64 | Apple Darwin (`x86_64-apple-darwin`) |
| macOS | aarch64 | Apple Silicon (`aarch64-apple-darwin`) |
| Windows | x86_64 | MSVC (`x86_64-pc-windows-msvc`) |
| Windows | aarch64 | MSVC (`aarch64-pc-windows-msvc`) |
| Browser | wasm32 | WebAssembly (`wasm32-unknown-unknown`) |

---

## 6. Language Bindings

Seven distribution packages cover eight target languages:

| Language | Binding Mechanism | Distribution Registry |
|---|---|---|
| **Rust** | Direct crate dependency (`core`, `actos`) | [crates.io](https://crates.io) |
| **Python** | PyO3 + `abi3` wheels | [PyPI](https://pypi.org) |
| **Node.js** | napi-rs native addon | [npm](https://npmjs.com) |
| **Browser** | wasm-bindgen (packaged within npm package) | [npm](https://npmjs.com) |
| **Go** | purego (no CGO required; embedded binary extraction) | Go Modules |
| **Java / Kotlin** | Foreign Function & Memory API (FFM, JDK 22+) | Maven Central |
| **C# (.NET)** | P/Invoke (`runtimes/{rid}/native/`) | NuGet |

---

## 7. C ABI Surface

The C header is located at [`abi/include/markstone.h`](abi/include/markstone.h).

```c
typedef enum {
  MARKSTONE_OK                 = 0,
  MARKSTONE_ERR_NULL_ARGUMENT  = 1,
  MARKSTONE_ERR_INVALID_UTF8   = 2,
  MARKSTONE_ERR_INPUT_TOO_LARGE= 3,
  MARKSTONE_ERR_DEPTH_EXCEEDED = 4,
  MARKSTONE_ERR_INTERNAL       = 5
} markstone_status;

/* Generic family */
markstone_status markstone_to_html(const char *input, size_t input_len, char **out, size_t *out_len);
markstone_status markstone_to_ast(const char *input, size_t input_len, char **out, size_t *out_len);

/* Actos family */
markstone_status markstone_actos_to_html(const char *input, size_t input_len, char **out, size_t *out_len);
markstone_status markstone_actos_to_ast(const char *input, size_t input_len, char **out, size_t *out_len);

/* Memory and metadata */
void markstone_free(char *ptr, size_t len);
const char *markstone_version(void);
uint32_t markstone_ast_schema_version(void);
```

---

## 8. Development

### Prerequisites

- Rust 1.85.0+ (2024 edition support)

### Workspace Commands

```bash
# Check all crates in the workspace
cargo check --workspace

# Run unit and integration tests across the workspace
cargo test --workspace

# Format code
cargo fmt --all

# Run linter
cargo clippy --workspace --all-targets
```

---

## 9. License

This project is licensed under the [MIT License](LICENSE).
