package actos_test

import (
	"errors"
	"strings"
	"testing"

	"github.com/dethrandir/markstone/bindings/go"
	"github.com/dethrandir/markstone/bindings/go/actos"
)

func TestActosPackage(t *testing.T) {
	if actos.Version() != "0.1.0" {
		t.Fatalf("expected version 0.1.0, got %q", actos.Version())
	}
	if actos.ASTSchemaVersion != 1 {
		t.Fatalf("expected ASTSchemaVersion 1, got %d", actos.ASTSchemaVersion)
	}

	input := "Mention @alice and #topic"
	html, err := actos.ToHTML(input)
	if err != nil {
		t.Fatalf("ToHTML failed: %v", err)
	}
	if !strings.Contains(html, `class="mention"`) || !strings.Contains(html, `class="tag"`) {
		t.Fatalf("ToHTML missing mention/tag: %s", html)
	}

	htmlBytes, err := actos.ToHTMLBytes([]byte(input))
	if err != nil {
		t.Fatalf("ToHTMLBytes failed: %v", err)
	}
	if htmlBytes != html {
		t.Fatalf("ToHTMLBytes != ToHTML")
	}

	ast, err := actos.ToAST(input)
	if err != nil {
		t.Fatalf("ToAST failed: %v", err)
	}
	if !strings.Contains(ast, `"type":"mention"`) {
		t.Fatalf("ToAST missing mention node: %s", ast)
	}

	astBytes, err := actos.ToASTBytes([]byte(input))
	if err != nil {
		t.Fatalf("ToASTBytes failed: %v", err)
	}
	if astBytes != ast {
		t.Fatalf("ToASTBytes != ToAST")
	}

	// Errors
	tooLarge := strings.Repeat("x", 4*1024*1024+1)
	if _, err := actos.ToHTML(tooLarge); !errors.Is(err, markstone.ErrInputTooLarge) {
		t.Fatalf("expected ErrInputTooLarge, got: %v", err)
	}
}
