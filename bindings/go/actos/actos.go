package actos

import "github.com/actos-dev/markstone/bindings/go"

// ASTSchemaVersion is the AST JSON schema version. Increments when the schema breaks.
const ASTSchemaVersion = markstone.ASTSchemaVersion

// Version returns the semver version string of the markstone native core.
func Version() string {
	return markstone.Version()
}

// ToHTML converts Markdown string to safe HTML including Actos mentions and tags.
func ToHTML(input string) (string, error) {
	return markstone.ActosToHTML(input)
}

// ToHTMLBytes converts Markdown byte slice to safe HTML including Actos mentions and tags.
func ToHTMLBytes(input []byte) (string, error) {
	return markstone.ActosToHTMLBytes(input)
}

// ToAST converts Markdown string to AST JSON including Actos mentions and tags.
func ToAST(input string) (string, error) {
	return markstone.ActosToAST(input)
}

// ToASTBytes converts Markdown byte slice to AST JSON including Actos mentions and tags.
func ToASTBytes(input []byte) (string, error) {
	return markstone.ActosToASTBytes(input)
}
