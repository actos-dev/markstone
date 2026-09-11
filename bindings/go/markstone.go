package markstone

import (
	"runtime"
	"unsafe"

	"github.com/dethrandir/markstone/bindings/go/internal/loader"
)

// PackageVersion is the semver version string of markstone.
const PackageVersion = "0.1.0"

// ASTSchemaVersion is the AST JSON schema version. Increments when the schema breaks.
const ASTSchemaVersion = 1

// Version returns the semver version string of the markstone native core.
func Version() string {
	abi, err := loader.GetABI()
	if err != nil {
		return ""
	}
	ptr := abi.Version()
	if ptr == 0 {
		return ""
	}
	p := (*byte)(*(*unsafe.Pointer)(unsafe.Pointer(&ptr)))
	var length int
	for *(*byte)(unsafe.Add(unsafe.Pointer(p), length)) != 0 {
		length++
	}
	return string(unsafe.Slice(p, length))
}

// ASTSchemaVersionNative returns the AST schema version reported by the native ABI.
func ASTSchemaVersionNative() uint32 {
	abi, err := loader.GetABI()
	if err != nil {
		return 0
	}
	return abi.ASTSchemaVersion()
}

func convertString(
	fn func(uintptr, uint, *uintptr, *uint) int32,
	freeFn func(uintptr, uint),
	input string,
) (string, error) {
	var inPtr uintptr
	inLen := uint(len(input))
	if inLen > 0 {
		inPtr = uintptr(unsafe.Pointer(unsafe.StringData(input)))
	}

	var out uintptr
	var outLen uint
	status := fn(inPtr, inLen, &out, &outLen)
	runtime.KeepAlive(input)

	if status != 0 {
		return "", statusToError(status)
	}

	defer freeFn(out, outLen)

	if out == 0 || outLen == 0 {
		return "", nil
	}

	ptr := *(*unsafe.Pointer)(unsafe.Pointer(&out))
	outBytes := unsafe.Slice((*byte)(ptr), outLen)
	return string(outBytes), nil
}

func convertBytes(
	fn func(uintptr, uint, *uintptr, *uint) int32,
	freeFn func(uintptr, uint),
	input []byte,
) (string, error) {
	var inPtr uintptr
	inLen := uint(len(input))
	if inLen > 0 {
		inPtr = uintptr(unsafe.Pointer(unsafe.SliceData(input)))
	}

	var out uintptr
	var outLen uint
	status := fn(inPtr, inLen, &out, &outLen)
	runtime.KeepAlive(input)

	if status != 0 {
		return "", statusToError(status)
	}

	defer freeFn(out, outLen)

	if out == 0 || outLen == 0 {
		return "", nil
	}

	ptr := *(*unsafe.Pointer)(unsafe.Pointer(&out))
	outBytes := unsafe.Slice((*byte)(ptr), outLen)
	return string(outBytes), nil
}

// ToHTML converts Markdown string to safe HTML using generic CommonMark + GFM pipeline.
func ToHTML(input string) (string, error) {
	abi, err := loader.GetABI()
	if err != nil {
		return "", err
	}
	return convertString(abi.ToHTML, abi.Free, input)
}

// ToHTMLBytes converts Markdown byte slice to safe HTML using generic CommonMark + GFM pipeline.
func ToHTMLBytes(input []byte) (string, error) {
	abi, err := loader.GetABI()
	if err != nil {
		return "", err
	}
	return convertBytes(abi.ToHTML, abi.Free, input)
}

// ToAST converts Markdown string to AST JSON using generic CommonMark + GFM pipeline.
func ToAST(input string) (string, error) {
	abi, err := loader.GetABI()
	if err != nil {
		return "", err
	}
	return convertString(abi.ToAST, abi.Free, input)
}

// ToASTBytes converts Markdown byte slice to AST JSON using generic CommonMark + GFM pipeline.
func ToASTBytes(input []byte) (string, error) {
	abi, err := loader.GetABI()
	if err != nil {
		return "", err
	}
	return convertBytes(abi.ToAST, abi.Free, input)
}

// ActosToHTML converts Markdown string to safe HTML including Actos mentions and tags.
func ActosToHTML(input string) (string, error) {
	abi, err := loader.GetABI()
	if err != nil {
		return "", err
	}
	return convertString(abi.ActosToHTML, abi.Free, input)
}

// ActosToHTMLBytes converts Markdown byte slice to safe HTML including Actos mentions and tags.
func ActosToHTMLBytes(input []byte) (string, error) {
	abi, err := loader.GetABI()
	if err != nil {
		return "", err
	}
	return convertBytes(abi.ActosToHTML, abi.Free, input)
}

// ActosToAST converts Markdown string to AST JSON including Actos mentions and tags.
func ActosToAST(input string) (string, error) {
	abi, err := loader.GetABI()
	if err != nil {
		return "", err
	}
	return convertString(abi.ActosToAST, abi.Free, input)
}

// ActosToASTBytes converts Markdown byte slice to AST JSON including Actos mentions and tags.
func ActosToASTBytes(input []byte) (string, error) {
	abi, err := loader.GetABI()
	if err != nil {
		return "", err
	}
	return convertBytes(abi.ActosToAST, abi.Free, input)
}
