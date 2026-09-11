package loader_test

import (
	"crypto/sha256"
	"encoding/hex"
	"os"
	"path/filepath"
	"testing"

	"github.com/dethrandir/markstone/bindings/go/internal/embeds"
	"github.com/dethrandir/markstone/bindings/go/internal/loader"
)

func TestLoaderGetABI(t *testing.T) {
	abi, err := loader.GetABI()
	if err != nil {
		t.Fatalf("loader.GetABI failed: %v", err)
	}
	if abi == nil {
		t.Fatal("expected non-nil abi")
	}
	if abi.Handle == 0 {
		t.Fatal("expected non-zero library handle")
	}
	if abi.Version == nil || abi.ToHTML == nil || abi.Free == nil {
		t.Fatal("expected function pointers to be populated")
	}
}

func TestLoaderWithEnvVar(t *testing.T) {
	// Find target/release/libmarkstone_abi.so
	dir, err := os.Getwd()
	if err != nil {
		t.Fatalf("Getwd failed: %v", err)
	}
	for i := 0; i < 10 && dir != "" && dir != "/" && dir != "."; i++ {
		candidate := filepath.Join(dir, "target", "release", "libmarkstone_abi.so")
		if _, err := os.Stat(candidate); err == nil {
			t.Setenv("MARKSTONE_LIBRARY_PATH", candidate)
			loader.ResetForTesting()
			abi, err := loader.GetABI()
			if err != nil {
				t.Fatalf("loader.GetABI with env var failed: %v", err)
			}
			if abi == nil {
				t.Fatal("expected non-nil abi from env var")
			}
			return
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			break
		}
		dir = parent
	}
}

func TestEmbeddedLibraryAndCacheExtraction(t *testing.T) {
	embedded, name := embeds.GetEmbeddedLibrary()
	if len(embedded) == 0 {
		t.Skip("no embedded library for current platform")
	}

	hashBytes := sha256.Sum256(embedded)
	hashStr := hex.EncodeToString(hashBytes[:])

	cacheDir, err := os.UserCacheDir()
	if err != nil {
		home, _ := os.UserHomeDir()
		cacheDir = filepath.Join(home, ".cache")
	}

	targetPath := filepath.Join(cacheDir, "markstone", loader.PackageVersion, hashStr, name)

	// Trigger load
	loader.ResetForTesting()
	abi, err := loader.GetABI()
	if err != nil {
		t.Fatalf("GetABI failed: %v", err)
	}
	if abi == nil {
		t.Fatal("abi is nil")
	}

	// Verify extracted file exists if extracted or can be extracted
	fi, err := os.Stat(targetPath)
	if err == nil {
		if fi.Size() != int64(len(embedded)) {
			t.Fatalf("cached file size mismatch: %d != %d", fi.Size(), len(embedded))
		}
		mode := fi.Mode().Perm()
		if mode&0111 == 0 {
			t.Fatalf("cached file is not executable: %v", mode)
		}
	}
}

func TestLoaderWithInvalidEnvVar(t *testing.T) {
	t.Setenv("MARKSTONE_LIBRARY_PATH", "/non/existent/path/libmarkstone_abi.so")
	loader.ResetForTesting()
	abi, err := loader.GetABI()
	if err == nil {
		t.Fatal("expected error with invalid MARKSTONE_LIBRARY_PATH")
	}
	if abi != nil {
		t.Fatal("expected nil abi when loading fails")
	}
}

func TestLoaderExtractDirectly(t *testing.T) {
	// Temporarily override MARKSTONE_LIBRARY_PATH to empty
	t.Setenv("MARKSTONE_LIBRARY_PATH", "")

	// Extract to a temp cache directory
	tmpCache, err := os.MkdirTemp("", "markstone-cache-test-*")
	if err != nil {
		t.Fatalf("failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpCache)

	t.Setenv("XDG_CACHE_HOME", tmpCache)

	loader.ResetForTesting()
	abi, err := loader.GetABI()
	if err != nil {
		t.Fatalf("GetABI failed: %v", err)
	}
	if abi == nil {
		t.Fatal("expected non-nil abi")
	}
}
