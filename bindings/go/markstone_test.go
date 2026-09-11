package markstone_test

import (
	"bytes"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"testing"

	"github.com/dethrandir/markstone/bindings/go"
)

func findCasesDir(t *testing.T) string {
	dir, err := os.Getwd()
	if err != nil {
		t.Fatalf("failed to get current directory: %v", err)
	}

	for i := 0; i < 10 && dir != "" && dir != "/" && dir != "."; i++ {
		candidate := filepath.Join(dir, "conformance", "cases")
		if fi, err := os.Stat(candidate); err == nil && fi.IsDir() {
			return candidate
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			break
		}
		dir = parent
	}

	t.Fatal("could not find conformance/cases directory")
	return ""
}

// TestConformance77Cases validates all 77 cases (308 checks) asserting byte-for-byte identity against golden files.
func TestConformance77Cases(t *testing.T) {
	casesDir := findCasesDir(t)
	entries, err := os.ReadDir(casesDir)
	if err != nil {
		t.Fatalf("failed to read cases dir: %v", err)
	}

	type testCase struct {
		name     string
		dir      string
		input    []byte
		genHTML  []byte
		genAST   []byte
		actoHTML []byte
		actoAST  []byte
	}

	var cases []testCase
	for _, entry := range entries {
		if !entry.IsDir() {
			continue
		}
		cDir := filepath.Join(casesDir, entry.Name())
		inputPath := filepath.Join(cDir, "input.md")
		input, err := os.ReadFile(inputPath)
		if err != nil {
			continue
		}

		genHTML, err := os.ReadFile(filepath.Join(cDir, "generic.html"))
		if err != nil {
			t.Fatalf("missing generic.html in %s", entry.Name())
		}
		genAST, err := os.ReadFile(filepath.Join(cDir, "generic.ast.json"))
		if err != nil {
			t.Fatalf("missing generic.ast.json in %s", entry.Name())
		}
		actoHTML, err := os.ReadFile(filepath.Join(cDir, "actos.html"))
		if err != nil {
			t.Fatalf("missing actos.html in %s", entry.Name())
		}
		actoAST, err := os.ReadFile(filepath.Join(cDir, "actos.ast.json"))
		if err != nil {
			t.Fatalf("missing actos.ast.json in %s", entry.Name())
		}

		cases = append(cases, testCase{
			name:     entry.Name(),
			dir:      cDir,
			input:    input,
			genHTML:  genHTML,
			genAST:   genAST,
			actoHTML: actoHTML,
			actoAST:  actoAST,
		})
	}

	if len(cases) != 77 {
		t.Fatalf("expected 77 test cases, found %d", len(cases))
	}

	totalChecks := 0

	for _, tc := range cases {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			inputStr := string(tc.input)

			// 1. generic-html
			t.Run("generic-html", func(t *testing.T) {
				actualStr, err := markstone.ToHTML(inputStr)
				if err != nil {
					t.Fatalf("ToHTML failed: %v", err)
				}
				if actualStr != string(tc.genHTML) {
					t.Fatalf("generic-html mismatch in %s:\nexpected: %q\ngot:      %q", tc.name, string(tc.genHTML), actualStr)
				}

				actualBytes, err := markstone.ToHTMLBytes(tc.input)
				if err != nil {
					t.Fatalf("ToHTMLBytes failed: %v", err)
				}
				if actualBytes != string(tc.genHTML) {
					t.Fatalf("ToHTMLBytes mismatch in %s", tc.name)
				}
			})
			totalChecks++

			// 2. generic-ast
			t.Run("generic-ast", func(t *testing.T) {
				actualStr, err := markstone.ToAST(inputStr)
				if err != nil {
					t.Fatalf("ToAST failed: %v", err)
				}
				if actualStr != string(tc.genAST) {
					t.Fatalf("generic-ast mismatch in %s:\nexpected: %q\ngot:      %q", tc.name, string(tc.genAST), actualStr)
				}

				actualBytes, err := markstone.ToASTBytes(tc.input)
				if err != nil {
					t.Fatalf("ToASTBytes failed: %v", err)
				}
				if actualBytes != string(tc.genAST) {
					t.Fatalf("ToASTBytes mismatch in %s", tc.name)
				}
			})
			totalChecks++

			// 3. actos-html
			t.Run("actos-html", func(t *testing.T) {
				actualStr, err := markstone.ActosToHTML(inputStr)
				if err != nil {
					t.Fatalf("ActosToHTML failed: %v", err)
				}
				if actualStr != string(tc.actoHTML) {
					t.Fatalf("actos-html mismatch in %s:\nexpected: %q\ngot:      %q", tc.name, string(tc.actoHTML), actualStr)
				}

				actualBytes, err := markstone.ActosToHTMLBytes(tc.input)
				if err != nil {
					t.Fatalf("ActosToHTMLBytes failed: %v", err)
				}
				if actualBytes != string(tc.actoHTML) {
					t.Fatalf("ActosToHTMLBytes mismatch in %s", tc.name)
				}
			})
			totalChecks++

			// 4. actos-ast
			t.Run("actos-ast", func(t *testing.T) {
				actualStr, err := markstone.ActosToAST(inputStr)
				if err != nil {
					t.Fatalf("ActosToAST failed: %v", err)
				}
				if actualStr != string(tc.actoAST) {
					t.Fatalf("actos-ast mismatch in %s:\nexpected: %q\ngot:      %q", tc.name, string(tc.actoAST), actualStr)
				}

				actualBytes, err := markstone.ActosToASTBytes(tc.input)
				if err != nil {
					t.Fatalf("ActosToASTBytes failed: %v", err)
				}
				if actualBytes != string(tc.actoAST) {
					t.Fatalf("ActosToASTBytes mismatch in %s", tc.name)
				}
			})
			totalChecks++
		})
	}

	if totalChecks != 308 {
		t.Fatalf("expected 308 checks, performed %d", totalChecks)
	}
}

// TestVersionAndSchema tests Version and ASTSchemaVersion.
func TestVersionAndSchema(t *testing.T) {
	v := markstone.Version()
	if v != "0.1.0" {
		t.Fatalf("expected version 0.1.0, got %q", v)
	}

	if markstone.ASTSchemaVersion != 1 {
		t.Fatalf("expected ASTSchemaVersion == 1, got %d", markstone.ASTSchemaVersion)
	}

	if markstone.ASTSchemaVersionNative() != 1 {
		t.Fatalf("expected native ASTSchemaVersion == 1, got %d", markstone.ASTSchemaVersionNative())
	}
}

// TestInputTooLarge verifies input size exceeding 4 MiB returns ErrInputTooLarge.
func TestInputTooLarge(t *testing.T) {
	limit := 4 * 1024 * 1024
	tooLarge := strings.Repeat("x", limit+1)
	tooLargeBytes := []byte(tooLarge)

	// String functions
	if _, err := markstone.ToHTML(tooLarge); !errors.Is(err, markstone.ErrInputTooLarge) {
		t.Fatalf("expected ErrInputTooLarge for ToHTML, got: %v", err)
	}
	if _, err := markstone.ToAST(tooLarge); !errors.Is(err, markstone.ErrInputTooLarge) {
		t.Fatalf("expected ErrInputTooLarge for ToAST, got: %v", err)
	}
	if _, err := markstone.ActosToHTML(tooLarge); !errors.Is(err, markstone.ErrInputTooLarge) {
		t.Fatalf("expected ErrInputTooLarge for ActosToHTML, got: %v", err)
	}
	if _, err := markstone.ActosToAST(tooLarge); !errors.Is(err, markstone.ErrInputTooLarge) {
		t.Fatalf("expected ErrInputTooLarge for ActosToAST, got: %v", err)
	}

	// Byte functions
	if _, err := markstone.ToHTMLBytes(tooLargeBytes); !errors.Is(err, markstone.ErrInputTooLarge) {
		t.Fatalf("expected ErrInputTooLarge for ToHTMLBytes, got: %v", err)
	}
	if _, err := markstone.ToASTBytes(tooLargeBytes); !errors.Is(err, markstone.ErrInputTooLarge) {
		t.Fatalf("expected ErrInputTooLarge for ToASTBytes, got: %v", err)
	}
	if _, err := markstone.ActosToHTMLBytes(tooLargeBytes); !errors.Is(err, markstone.ErrInputTooLarge) {
		t.Fatalf("expected ErrInputTooLarge for ActosToHTMLBytes, got: %v", err)
	}
	if _, err := markstone.ActosToASTBytes(tooLargeBytes); !errors.Is(err, markstone.ErrInputTooLarge) {
		t.Fatalf("expected ErrInputTooLarge for ActosToASTBytes, got: %v", err)
	}
}

// TestDepthExceeded verifies block nesting depth > 64 returns ErrDepthExceeded.
func TestDepthExceeded(t *testing.T) {
	// 64 blockquotes + 1 paragraph = depth 65 > 64 limit
	tooDeep := strings.Repeat("> ", 65) + "nested\n"
	tooDeepBytes := []byte(tooDeep)

	if _, err := markstone.ToHTML(tooDeep); !errors.Is(err, markstone.ErrDepthExceeded) {
		t.Fatalf("expected ErrDepthExceeded for ToHTML, got: %v", err)
	}
	if _, err := markstone.ToAST(tooDeep); !errors.Is(err, markstone.ErrDepthExceeded) {
		t.Fatalf("expected ErrDepthExceeded for ToAST, got: %v", err)
	}
	if _, err := markstone.ActosToHTML(tooDeep); !errors.Is(err, markstone.ErrDepthExceeded) {
		t.Fatalf("expected ErrDepthExceeded for ActosToHTML, got: %v", err)
	}
	if _, err := markstone.ActosToAST(tooDeep); !errors.Is(err, markstone.ErrDepthExceeded) {
		t.Fatalf("expected ErrDepthExceeded for ActosToAST, got: %v", err)
	}

	if _, err := markstone.ToHTMLBytes(tooDeepBytes); !errors.Is(err, markstone.ErrDepthExceeded) {
		t.Fatalf("expected ErrDepthExceeded for ToHTMLBytes, got: %v", err)
	}
	if _, err := markstone.ToASTBytes(tooDeepBytes); !errors.Is(err, markstone.ErrDepthExceeded) {
		t.Fatalf("expected ErrDepthExceeded for ToASTBytes, got: %v", err)
	}
	if _, err := markstone.ActosToHTMLBytes(tooDeepBytes); !errors.Is(err, markstone.ErrDepthExceeded) {
		t.Fatalf("expected ErrDepthExceeded for ActosToHTMLBytes, got: %v", err)
	}
	if _, err := markstone.ActosToASTBytes(tooDeepBytes); !errors.Is(err, markstone.ErrDepthExceeded) {
		t.Fatalf("expected ErrDepthExceeded for ActosToASTBytes, got: %v", err)
	}

	// Valid depth (63 blockquotes + 1 paragraph = depth 64 <= 64)
	validDepth := strings.Repeat("> ", 63) + "ok content\n"
	out, err := markstone.ToHTML(validDepth)
	if err != nil {
		t.Fatalf("expected depth 64 to succeed, got: %v", err)
	}
	if !strings.Contains(out, "ok content") {
		t.Fatalf("missing content in valid depth output: %s", out)
	}
}

// TestInvalidUTF8 verifies invalid UTF-8 bytes return ErrInvalidUTF8.
func TestInvalidUTF8(t *testing.T) {
	invalidBytes := []byte{0xff, 0xfe, 0xfd}

	if _, err := markstone.ToHTMLBytes(invalidBytes); !errors.Is(err, markstone.ErrInvalidUTF8) {
		t.Fatalf("expected ErrInvalidUTF8 for ToHTMLBytes, got: %v", err)
	}
	if _, err := markstone.ToASTBytes(invalidBytes); !errors.Is(err, markstone.ErrInvalidUTF8) {
		t.Fatalf("expected ErrInvalidUTF8 for ToASTBytes, got: %v", err)
	}
	if _, err := markstone.ActosToHTMLBytes(invalidBytes); !errors.Is(err, markstone.ErrInvalidUTF8) {
		t.Fatalf("expected ErrInvalidUTF8 for ActosToHTMLBytes, got: %v", err)
	}
	if _, err := markstone.ActosToASTBytes(invalidBytes); !errors.Is(err, markstone.ErrInvalidUTF8) {
		t.Fatalf("expected ErrInvalidUTF8 for ActosToASTBytes, got: %v", err)
	}

	// String containing invalid UTF-8
	invalidStr := string(invalidBytes)
	if _, err := markstone.ToHTML(invalidStr); !errors.Is(err, markstone.ErrInvalidUTF8) {
		t.Fatalf("expected ErrInvalidUTF8 for ToHTML, got: %v", err)
	}
}

// TestConcurrentCalls verifies thread safety under 20 concurrent goroutines.
func TestConcurrentCalls(t *testing.T) {
	goroutines := 20
	iterations := 100
	var wg sync.WaitGroup
	errs := make(chan error, goroutines*iterations)

	for g := 0; g < goroutines; g++ {
		wg.Add(1)
		go func(gid int) {
			defer wg.Done()
			for i := 0; i < iterations; i++ {
				input := fmt.Sprintf("## Goroutine %d - iteration %d\n\nHello @alice and #tag!", gid, i)

				html, err := markstone.ToHTML(input)
				if err != nil {
					errs <- fmt.Errorf("ToHTML error: %w", err)
					return
				}
				if !strings.HasPrefix(html, fmt.Sprintf("<h2>Goroutine %d - iteration %d</h2>", gid, i)) {
					errs <- fmt.Errorf("ToHTML unexpected output: %s", html)
					return
				}

				actosHtml, err := markstone.ActosToHTML(input)
				if err != nil {
					errs <- fmt.Errorf("ActosToHTML error: %w", err)
					return
				}
				if !strings.Contains(actosHtml, `class="mention"`) || !strings.Contains(actosHtml, `class="tag"`) {
					errs <- fmt.Errorf("ActosToHTML missing mention/tag: %s", actosHtml)
					return
				}

				ast, err := markstone.ToAST(input)
				if err != nil {
					errs <- fmt.Errorf("ToAST error: %w", err)
					return
				}
				if !strings.Contains(ast, `"schema":1`) {
					errs <- fmt.Errorf("ToAST invalid schema: %s", ast)
					return
				}

				actosAst, err := markstone.ActosToAST(input)
				if err != nil {
					errs <- fmt.Errorf("ActosToAST error: %w", err)
					return
				}
				if !strings.Contains(actosAst, `"type":"mention"`) {
					errs <- fmt.Errorf("ActosToAST missing mention node: %s", actosAst)
					return
				}
			}
		}(g)
	}

	wg.Wait()
	close(errs)

	for err := range errs {
		t.Fatal(err)
	}
}

// TestMemoryStability verifies memory remains stable across thousands of conversions without leaks.
func TestMemoryStability(t *testing.T) {
	iterations := 5000
	input := "### Repeated memory stability test\n\nParagraph with **bold**, *italic*, and `code`."

	for i := 0; i < iterations; i++ {
		html, err := markstone.ToHTML(input)
		if err != nil {
			t.Fatalf("ToHTML iteration %d failed: %v", i, err)
		}
		if !strings.Contains(html, "<h3>Repeated memory stability test</h3>") {
			t.Fatalf("unexpected HTML in iteration %d", i)
		}
	}
}

// TestEmptyAndWhitespace verifies handling of empty inputs and whitespace.
func TestEmptyAndWhitespace(t *testing.T) {
	html, err := markstone.ToHTML("")
	if err != nil {
		t.Fatalf("ToHTML empty failed: %v", err)
	}
	if html != "" {
		t.Fatalf("expected empty HTML, got %q", html)
	}

	ast, err := markstone.ToAST("")
	if err != nil {
		t.Fatalf("ToAST empty failed: %v", err)
	}
	if !strings.Contains(ast, `"schema":1`) || !strings.Contains(ast, `"type":"document"`) {
		t.Fatalf("unexpected AST for empty input: %s", ast)
	}

	// Bytes
	htmlBytes, err := markstone.ToHTMLBytes([]byte{})
	if err != nil {
		t.Fatalf("ToHTMLBytes empty failed: %v", err)
	}
	if htmlBytes != "" {
		t.Fatalf("expected empty HTML for empty bytes, got %q", htmlBytes)
	}

	// Nil byte slice
	htmlNil, err := markstone.ToHTMLBytes(nil)
	if err != nil {
		t.Fatalf("ToHTMLBytes nil failed: %v", err)
	}
	if htmlNil != "" {
		t.Fatalf("expected empty HTML for nil slice, got %q", htmlNil)
	}
}

// TestActosMentionsAndTags verifies mention and tag linking behavior.
func TestActosMentionsAndTags(t *testing.T) {
	input := "Hello @alice and @bob_123, check #rust-lang and #v1!"

	genericHtml, err := markstone.ToHTML(input)
	if err != nil {
		t.Fatalf("ToHTML failed: %v", err)
	}
	if strings.Contains(genericHtml, `<a href="/u/alice"`) {
		t.Fatalf("generic HTML should not link mentions: %s", genericHtml)
	}

	actosHtml, err := markstone.ActosToHTML(input)
	if err != nil {
		t.Fatalf("ActosToHTML failed: %v", err)
	}
	if !strings.Contains(actosHtml, `<a href="/u/alice" class="mention">@alice</a>`) {
		t.Fatalf("actos HTML missing @alice link: %s", actosHtml)
	}
	if !strings.Contains(actosHtml, `<a href="/t/rust-lang" class="tag">#rust-lang</a>`) {
		t.Fatalf("actos HTML missing #rust-lang link: %s", actosHtml)
	}
}

// TestUnicodeAndEmbeddedNul tests unicode handling and embedded NUL characters.
func TestUnicodeAndEmbeddedNul(t *testing.T) {
	input := "# 🌍 Unicode & 1 < 2 & 3 > 2 \n\n日本語 text, العربية, and `<code>`.\n"
	html, err := markstone.ToHTML(input)
	if err != nil {
		t.Fatalf("ToHTML failed: %v", err)
	}
	if !strings.Contains(html, "<h1>🌍 Unicode &amp; 1 &lt; 2 &amp; 3 &gt; 2</h1>") {
		t.Fatalf("HTML escaped incorrectly: %s", html)
	}
	if !strings.Contains(html, "日本語") || !strings.Contains(html, "العربية") {
		t.Fatalf("Unicode characters lost: %s", html)
	}

	// Embedded NUL byte is valid markdown text according to spec
	withNul := []byte("Hello\x00World")
	nulHtml, err := markstone.ToHTMLBytes(withNul)
	if err != nil {
		t.Fatalf("ToHTMLBytes with embedded NUL failed: %v", err)
	}
	if !bytes.Contains([]byte(nulHtml), []byte("World")) {
		t.Fatalf("output missing text after NUL: %s", nulHtml)
	}
}
