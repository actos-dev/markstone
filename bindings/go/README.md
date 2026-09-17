# markstone (Go)

Go bindings for markstone: fast, safe Markdown-to-HTML and AST engine with Actos extensions, powered by [purego](https://github.com/ebitengine/purego).

## Key Features

- **No CGO Required:** Built completely with `purego`. Works cleanly with `CGO_ENABLED=0` and does not break cross-compilation.
- **Embedded Native Core:** The platform-appropriate native library is bundled using `embed` and extracted automatically to `os.UserCacheDir()` on first use.
- **Full Conformance:** Produces 100% byte-for-byte identical output to the core Rust engine and all other language bindings.
- **Zero Configuration:** Drop-in library extraction with `MARKSTONE_LIBRARY_PATH` escape hatch for restricted or `noexec` environments.

## Installation

```bash
go get github.com/actos-dev/markstone/bindings/go
```

## Usage

```go
package main

import (
	"fmt"
	"log"

	"github.com/actos-dev/markstone/bindings/go"
	"github.com/actos-dev/markstone/bindings/go/actos"
)

func main() {
	input := "# Hello world\n\nParagraph text with @alice and #tag."

	// Generic CommonMark + GFM
	html, err := markstone.ToHTML(input)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println(html)

	ast, err := markstone.ToAST(input)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println(ast)

	// Actos extensions (@mention and #tag linking)
	actosHtml, err := actos.ToHTML(input)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println(actosHtml)

	// Metadata
	fmt.Println("Version:", markstone.Version())
	fmt.Println("AST Schema Version:", markstone.ASTSchemaVersion)
}
```

## Library Resolution & Configuration

The loader resolves the native library in the following order:

1. **`MARKSTONE_LIBRARY_PATH` environment variable**: If set, points directly to a pre-installed `libmarkstone_abi.so` (or `.dylib` / `.dll`), or directory containing it.
2. **Repository Build Output**: Searches up parent directories for `target/release/` or `target/debug/` shared libraries.
3. **Embedded Extraction**: Automatically extracts the embedded library into `filepath.Join(os.UserCacheDir(), "markstone", version, hash, "libmarkstone_abi.so")` with permissions `0755`. The file is extracted only once per content hash.
4. **Error Handling**: If the cache directory is mounted with `noexec` or loading fails, an error directs the user to set `MARKSTONE_LIBRARY_PATH`.

## Running Tests

```bash
CGO_ENABLED=0 go test -v ./...
```
