package loader

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"sync"

	"github.com/actos-dev/markstone/bindings/go/internal/embeds"
	"github.com/ebitengine/purego"
)

// PackageVersion matches the markstone release version.
const PackageVersion = "0.1.0"

// NativeABI holds function pointers bound via purego to the C ABI library.
type NativeABI struct {
	Handle           uintptr
	ToHTML           func(input uintptr, inputLen uint, out *uintptr, outLen *uint) int32
	ToAST            func(input uintptr, inputLen uint, out *uintptr, outLen *uint) int32
	ActosToHTML      func(input uintptr, inputLen uint, out *uintptr, outLen *uint) int32
	ActosToAST       func(input uintptr, inputLen uint, out *uintptr, outLen *uint) int32
	Free             func(ptr uintptr, len uint)
	Version          func() uintptr
	ASTSchemaVersion func() uint32
}

var (
	once    sync.Once
	abi     *NativeABI
	loadErr error
)

// ResetForTesting resets the loader singleton so tests can exercise different loader paths.
func ResetForTesting() {
	once = sync.Once{}
	abi = nil
	loadErr = nil
}

// GetABI returns the loaded NativeABI singleton, initializing it on first call.
func GetABI() (*NativeABI, error) {
	once.Do(func() {
		abi, loadErr = Load()
	})
	return abi, loadErr
}

// candidateFilenames returns platform-specific shared library names to probe.
func candidateFilenames() []string {
	switch runtime.GOOS {
	case "windows":
		return []string{"markstone_abi.dll", "markstone.dll"}
	case "darwin":
		return []string{"libmarkstone_abi.dylib", "libmarkstone.dylib"}
	default:
		return []string{"libmarkstone_abi.so", "libmarkstone.so"}
	}
}

func tryLoad(path string) (*NativeABI, error) {
	handle, err := purego.Dlopen(path, purego.RTLD_NOW|purego.RTLD_GLOBAL)
	if err != nil {
		return nil, err
	}
	return bind(handle)
}

func bind(handle uintptr) (n *NativeABI, err error) {
	defer func() {
		if r := recover(); r != nil {
			n = nil
			err = fmt.Errorf("markstone: failed to bind native symbols: %v", r)
		}
	}()

	res := &NativeABI{Handle: handle}
	purego.RegisterLibFunc(&res.ToHTML, handle, "markstone_to_html")
	purego.RegisterLibFunc(&res.ToAST, handle, "markstone_to_ast")
	purego.RegisterLibFunc(&res.ActosToHTML, handle, "markstone_actos_to_html")
	purego.RegisterLibFunc(&res.ActosToAST, handle, "markstone_actos_to_ast")
	purego.RegisterLibFunc(&res.Free, handle, "markstone_free")
	purego.RegisterLibFunc(&res.Version, handle, "markstone_version")
	purego.RegisterLibFunc(&res.ASTSchemaVersion, handle, "markstone_ast_schema_version")
	return res, nil
}

// Load executes the four-tier library resolution strategy:
// 1. Check MARKSTONE_LIBRARY_PATH environment variable.
// 2. Walk up directory tree probing repository build outputs (target/release, target/debug).
// 3. Extract embedded library to os.UserCacheDir() named by version and sha256 hash.
// 4. Return descriptive error directing user to MARKSTONE_LIBRARY_PATH if all fail.
func Load() (*NativeABI, error) {
	names := candidateFilenames()

	// 1. Check MARKSTONE_LIBRARY_PATH environment variable
	if envPath := os.Getenv("MARKSTONE_LIBRARY_PATH"); envPath != "" {
		fi, err := os.Stat(envPath)
		if err == nil {
			if fi.IsDir() {
				for _, name := range names {
					candidate := filepath.Join(envPath, name)
					if _, err := os.Stat(candidate); err == nil {
						if loaded, err := tryLoad(candidate); err == nil {
							return loaded, nil
						}
					}
				}
			} else {
				if loaded, err := tryLoad(envPath); err == nil {
					return loaded, nil
				} else {
					return nil, fmt.Errorf("markstone: failed to load library from MARKSTONE_LIBRARY_PATH=%s: %w", envPath, err)
				}
			}
		} else {
			return nil, fmt.Errorf("markstone: MARKSTONE_LIBRARY_PATH=%s does not exist: %w", envPath, err)
		}
	}

	// 2. Walk up directory tree probing repository build outputs
	dirsToProbe := []string{}
	if cwd, err := os.Getwd(); err == nil {
		dirsToProbe = append(dirsToProbe, cwd)
	}
	if exe, err := os.Executable(); err == nil {
		dirsToProbe = append(dirsToProbe, filepath.Dir(exe))
	}

	for _, startDir := range dirsToProbe {
		dir := startDir
		for i := 0; i < 10 && dir != "" && dir != "/" && dir != "."; i++ {
			for _, sub := range []string{filepath.Join("target", "release"), filepath.Join("target", "debug")} {
				for _, name := range names {
					candidate := filepath.Join(dir, sub, name)
					if _, err := os.Stat(candidate); err == nil {
						if loaded, err := tryLoad(candidate); err == nil {
							return loaded, nil
						}
					}
				}
			}
			parent := filepath.Dir(dir)
			if parent == dir {
				break
			}
			dir = parent
		}
	}

	// 3. Extract from embedded bytes into os.UserCacheDir()
	embedded, defaultName := embeds.GetEmbeddedLibrary()
	if len(embedded) > 0 {
		hashBytes := sha256.Sum256(embedded)
		hashStr := hex.EncodeToString(hashBytes[:])

		cacheDir, err := os.UserCacheDir()
		if err != nil {
			if home, hErr := os.UserHomeDir(); hErr == nil {
				cacheDir = filepath.Join(home, ".cache")
			} else {
				return nil, fmt.Errorf("markstone: could not determine user cache directory: %w. Please set MARKSTONE_LIBRARY_PATH to point to libmarkstone_abi.so", err)
			}
		}

		targetDir := filepath.Join(cacheDir, "markstone", PackageVersion, hashStr)
		targetFile := filepath.Join(targetDir, defaultName)

		// Check if the library is already extracted
		fi, err := os.Stat(targetFile)
		extracted := (err == nil && fi.Size() == int64(len(embedded)))

		if !extracted {
			if err := os.MkdirAll(targetDir, 0755); err != nil {
				return nil, fmt.Errorf("markstone: failed to create cache directory %s: %w. If the cache directory is not writable, please set MARKSTONE_LIBRARY_PATH", targetDir, err)
			}

			tmpFile, err := os.CreateTemp(targetDir, "libmarkstone-*.tmp")
			if err != nil {
				return nil, fmt.Errorf("markstone: failed to create temporary file in %s: %w. If the cache directory is not writable, please set MARKSTONE_LIBRARY_PATH", targetDir, err)
			}
			tmpPath := tmpFile.Name()

			if _, err := tmpFile.Write(embedded); err != nil {
				tmpFile.Close()
				os.Remove(tmpPath)
				return nil, fmt.Errorf("markstone: failed to write native library to %s: %w. Please set MARKSTONE_LIBRARY_PATH", tmpPath, err)
			}
			if err := tmpFile.Chmod(0755); err != nil {
				tmpFile.Close()
				os.Remove(tmpPath)
				return nil, fmt.Errorf("markstone: failed to chmod native library: %w", err)
			}
			if err := tmpFile.Close(); err != nil {
				os.Remove(tmpPath)
				return nil, fmt.Errorf("markstone: failed to close temporary native library file: %w", err)
			}

			if err := os.Rename(tmpPath, targetFile); err != nil {
				// Concurrent write or Windows file lock: verify targetFile exists
				if fi2, err2 := os.Stat(targetFile); err2 != nil || fi2.Size() != int64(len(embedded)) {
					os.Remove(tmpPath)
					return nil, fmt.Errorf("markstone: failed to install native library to %s: %w. Please set MARKSTONE_LIBRARY_PATH", targetFile, err)
				}
				os.Remove(tmpPath)
			}
		}

		loaded, err := tryLoad(targetFile)
		if err != nil {
			return nil, fmt.Errorf("markstone: failed to load native library from %s: %w. If the cache directory is mounted with noexec, please install the library to an executable location and set MARKSTONE_LIBRARY_PATH", targetFile, err)
		}
		return loaded, nil
	}

	return nil, fmt.Errorf("markstone: native library could not be located. Looked in MARKSTONE_LIBRARY_PATH, target/release, target/debug, and no embedded binary available for %s_%s. Please compile libmarkstone_abi and set MARKSTONE_LIBRARY_PATH", runtime.GOOS, runtime.GOARCH)
}
