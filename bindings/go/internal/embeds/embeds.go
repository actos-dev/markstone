package embeds

import (
	_ "embed"
	"runtime"
)

//go:embed libmarkstone_abi_linux_amd64.so
var libLinuxAmd64 []byte

// GetEmbeddedLibrary returns the embedded native library for the current runtime platform,
// along with its suggested filename. If no embedded library is available for this platform,
// it returns nil and an empty string.
func GetEmbeddedLibrary() ([]byte, string) {
	key := runtime.GOOS + "_" + runtime.GOARCH
	switch key {
	case "linux_amd64":
		return libLinuxAmd64, "libmarkstone_abi.so"
	default:
		return nil, ""
	}
}
