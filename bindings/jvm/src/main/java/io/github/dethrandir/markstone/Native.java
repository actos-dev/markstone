package io.github.dethrandir.markstone;

import java.io.IOException;
import java.io.InputStream;
import java.lang.foreign.AddressLayout;
import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.List;
import java.util.Locale;
import java.util.Objects;

/**
 * FFM plumbing for the markstone C ABI.
 *
 * <p>Loads the native library using a four-tier discovery strategy and binds
 * native entry points using {@link java.lang.foreign}.
 */
final class Native {

    private Native() {
    }

    private static final Linker LINKER = Linker.nativeLinker();
    private static final Arena LIBRARY_ARENA = Arena.ofShared();
    private static final SymbolLookup LOOKUP = loadLibrary();

    private static final ValueLayout.OfInt INT = ValueLayout.JAVA_INT;
    private static final ValueLayout.OfLong LONG = ValueLayout.JAVA_LONG;
    private static final AddressLayout PTR = ValueLayout.ADDRESS;

    private static final MethodHandle TO_HTML = downcall("markstone_to_html",
            FunctionDescriptor.of(INT, PTR, LONG, PTR, PTR));

    private static final MethodHandle TO_AST = downcall("markstone_to_ast",
            FunctionDescriptor.of(INT, PTR, LONG, PTR, PTR));

    private static final MethodHandle ACTOS_TO_HTML = downcall("markstone_actos_to_html",
            FunctionDescriptor.of(INT, PTR, LONG, PTR, PTR));

    private static final MethodHandle ACTOS_TO_AST = downcall("markstone_actos_to_ast",
            FunctionDescriptor.of(INT, PTR, LONG, PTR, PTR));

    private static final MethodHandle FREE = downcall("markstone_free",
            FunctionDescriptor.ofVoid(PTR, LONG));

    private static final MethodHandle VERSION = downcall("markstone_version",
            FunctionDescriptor.of(PTR));

    private static final MethodHandle AST_SCHEMA_VERSION = downcall("markstone_ast_schema_version",
            FunctionDescriptor.of(INT));

    private static MethodHandle downcall(String name, FunctionDescriptor descriptor) {
        MemorySegment symbol = LOOKUP.find(name).orElseThrow(
                () -> new UnsatisfiedLinkError("markstone: missing native symbol '" + name + "'"));
        return LINKER.downcallHandle(symbol, descriptor);
    }

    /**
     * Finds and loads the native library using the following priority:
     * 1. Check MARKSTONE_LIBRARY_PATH environment variable (file or directory).
     * 2. Probe repository build output (target/release and target/debug).
     * 3. Extract bundled library from classpath resources into user cache directory.
     * 4. Fallback to System.loadLibrary("markstone_abi") / System.loadLibrary("markstone").
     */
    private static SymbolLookup loadLibrary() {
        List<String> fileNames = libraryFileNames();

        // 1. Check MARKSTONE_LIBRARY_PATH environment variable
        String envPath = System.getenv("MARKSTONE_LIBRARY_PATH");
        if (envPath != null && !envPath.isBlank()) {
            Path candidate = Path.of(envPath);
            if (Files.isRegularFile(candidate)) {
                return SymbolLookup.libraryLookup(candidate, LIBRARY_ARENA);
            }
            for (String fileName : fileNames) {
                Path candidateFile = candidate.resolve(fileName);
                if (Files.isRegularFile(candidateFile)) {
                    return SymbolLookup.libraryLookup(candidateFile, LIBRARY_ARENA);
                }
            }
        }

        // 2. Probe repository build output
        Path dir = Path.of("").toAbsolutePath();
        for (int i = 0; i < 10 && dir != null; i++) {
            for (String sub : new String[] {"target/release", "target/debug"}) {
                for (String fileName : fileNames) {
                    Path candidate = dir.resolve(sub).resolve(fileName);
                    if (Files.isRegularFile(candidate)) {
                        return SymbolLookup.libraryLookup(candidate, LIBRARY_ARENA);
                    }
                }
            }
            dir = dir.getParent();
        }

        // 3. Extract bundled library from classpath resources
        for (String fileName : fileNames) {
            Path bundled = extractFromClasspath(fileName);
            if (bundled != null && Files.isRegularFile(bundled)) {
                return SymbolLookup.libraryLookup(bundled, LIBRARY_ARENA);
            }
        }

        // 4. Fallback to System.loadLibrary
        for (String libName : new String[] {"markstone_abi", "markstone"}) {
            try {
                System.loadLibrary(libName);
                return SymbolLookup.loaderLookup();
            } catch (UnsatisfiedLinkError ignored) {
            }
        }

        throw new UnsatisfiedLinkError(
                "markstone: could not find " + fileNames.get(0) + ". " +
                "Searched MARKSTONE_LIBRARY_PATH, repository target/release and target/debug, " +
                "bundled classpath resource native/" + platformDirectory() + "/" + fileNames.get(0) + ", " +
                "and System.loadLibrary(\"markstone_abi\").");
    }

    private static List<String> libraryFileNames() {
        String os = System.getProperty("os.name", "").toLowerCase(Locale.ROOT);
        if (os.contains("win")) {
            return List.of("markstone_abi.dll", "markstone.dll");
        }
        if (os.contains("mac") || os.contains("darwin")) {
            return List.of("libmarkstone_abi.dylib", "libmarkstone.dylib");
        }
        return List.of("libmarkstone_abi.so", "libmarkstone.so");
    }

    private static String platformDirectory() {
        String os = System.getProperty("os.name", "").toLowerCase(Locale.ROOT);
        String arch = System.getProperty("os.arch", "").toLowerCase(Locale.ROOT);

        String osName = os.contains("win") ? "windows"
                : (os.contains("mac") || os.contains("darwin")) ? "macos"
                : "linux";
        String archName = switch (arch) {
            case "amd64", "x86_64" -> "x86_64";
            case "aarch64", "arm64" -> "aarch64";
            default -> arch;
        };
        return osName + "-" + archName;
    }

    private static Path resolveCacheDir() {
        String xdg = System.getenv("XDG_CACHE_HOME");
        if (xdg != null && !xdg.isBlank()) {
            return Path.of(xdg);
        }
        String os = System.getProperty("os.name", "").toLowerCase(Locale.ROOT);
        String home = System.getProperty("user.home", ".");
        if (os.contains("mac") || os.contains("darwin")) {
            return Path.of(home, "Library", "Caches");
        }
        if (os.contains("win")) {
            String localAppData = System.getenv("LOCALAPPDATA");
            if (localAppData != null && !localAppData.isBlank()) {
                return Path.of(localAppData);
            }
            return Path.of(home, "AppData", "Local");
        }
        return Path.of(home, ".cache");
    }

    private static InputStream openResourceStream(String resource) {
        InputStream in = Native.class.getResourceAsStream(resource);
        if (in != null) {
            return in;
        }
        if (resource.startsWith("/")) {
            in = Native.class.getResourceAsStream(resource.substring(1));
            if (in != null) {
                return in;
            }
        }
        ClassLoader cl = Thread.currentThread().getContextClassLoader();
        if (cl != null) {
            String rel = resource.startsWith("/") ? resource.substring(1) : resource;
            in = cl.getResourceAsStream(rel);
            if (in != null) {
                return in;
            }
        }
        return null;
    }

    private static Path extractFromClasspath(String fileName) {
        String platformDir = platformDirectory();
        String resource = "/native/" + platformDir + "/" + fileName;
        InputStream in = openResourceStream(resource);
        if (in == null) {
            return null;
        }

        try (in) {
            Path cacheDir = resolveCacheDir();
            Path targetDir = cacheDir.resolve("markstone").resolve("0.1.0").resolve(platformDir);
            Files.createDirectories(targetDir);
            Path targetFile = targetDir.resolve(fileName);

            if (Files.isRegularFile(targetFile) && Files.size(targetFile) > 0) {
                return targetFile;
            }

            Path temp = Files.createTempFile(targetDir, "libmarkstone", ".tmp");
            try {
                Files.copy(in, temp, StandardCopyOption.REPLACE_EXISTING);
                try {
                    Files.move(temp, targetFile, StandardCopyOption.REPLACE_EXISTING, StandardCopyOption.ATOMIC_MOVE);
                } catch (IOException e) {
                    Files.move(temp, targetFile, StandardCopyOption.REPLACE_EXISTING);
                }
                return targetFile;
            } finally {
                Files.deleteIfExists(temp);
            }
        } catch (IOException e) {
            // Fallback to system temp file
            InputStream fallbackIn = openResourceStream(resource);
            if (fallbackIn != null) {
                try (fallbackIn) {
                    Path temp = Files.createTempFile("markstone_abi", fileName);
                    temp.toFile().deleteOnExit();
                    Files.copy(fallbackIn, temp, StandardCopyOption.REPLACE_EXISTING);
                    return temp;
                } catch (IOException ignored) {
                }
            }
            throw new UnsatisfiedLinkError(
                    "markstone: failed to unpack native library from " + resource + ": " + e.getMessage());
        }
    }

    private static void checkStatus(int status) {
        switch (status) {
            case 0:
                return;
            case 1:
                throw new IllegalArgumentException("markstone: null argument passed to native function");
            case 2:
                throw new InvalidUtf8Exception("markstone: input contains invalid UTF-8");
            case 3:
                throw new InputTooLargeException("markstone: input size exceeds 4 MiB limit");
            case 4:
                throw new DepthExceededException("markstone: block nesting depth exceeds limit of 64");
            case 5:
                throw new MarkstoneException("markstone: internal error or caught panic");
            default:
                throw new MarkstoneException("markstone: unknown error status " + status);
        }
    }

    private static String convert(MethodHandle handle, String input) {
        Objects.requireNonNull(input, "input must not be null");
        byte[] utf8Bytes = input.getBytes(StandardCharsets.UTF_8);
        long inputLen = utf8Bytes.length;

        try (Arena arena = Arena.ofConfined()) {
            MemorySegment inputSegment = utf8Bytes.length > 0
                    ? arena.allocateFrom(ValueLayout.JAVA_BYTE, utf8Bytes)
                    : MemorySegment.NULL;
            MemorySegment outPtr = arena.allocate(ValueLayout.ADDRESS);
            MemorySegment outLenPtr = arena.allocate(ValueLayout.JAVA_LONG);

            int status;
            try {
                status = (int) handle.invokeExact(inputSegment, inputLen, outPtr, outLenPtr);
            } catch (Throwable t) {
                throw wrap(t);
            }

            checkStatus(status);

            MemorySegment resultAddress = outPtr.get(ValueLayout.ADDRESS, 0);
            long resultLen = outLenPtr.get(ValueLayout.JAVA_LONG, 0);

            try {
                if (resultAddress.equals(MemorySegment.NULL) || resultLen == 0) {
                    return "";
                }
                MemorySegment slice = resultAddress.reinterpret(resultLen);
                byte[] outputBytes = new byte[(int) resultLen];
                MemorySegment.copy(slice, ValueLayout.JAVA_BYTE, 0, outputBytes, 0, (int) resultLen);
                return new String(outputBytes, StandardCharsets.UTF_8);
            } finally {
                free(resultAddress, resultLen);
            }
        }
    }

    private static void free(MemorySegment ptr, long len) {
        if (ptr != null && !ptr.equals(MemorySegment.NULL)) {
            try {
                FREE.invokeExact(ptr, len);
            } catch (Throwable t) {
                throw wrap(t);
            }
        }
    }

    static String toHtml(String input) {
        return convert(TO_HTML, input);
    }

    static String toAst(String input) {
        return convert(TO_AST, input);
    }

    static String actosToHtml(String input) {
        return convert(ACTOS_TO_HTML, input);
    }

    static String actosToAst(String input) {
        return convert(ACTOS_TO_AST, input);
    }

    static String version() {
        try {
            MemorySegment ptr = (MemorySegment) VERSION.invokeExact();
            if (ptr == null || ptr.equals(MemorySegment.NULL)) {
                return "";
            }
            return ptr.reinterpret(Long.MAX_VALUE).getString(0, StandardCharsets.UTF_8);
        } catch (Throwable t) {
            throw wrap(t);
        }
    }

    static int astSchemaVersion() {
        try {
            return (int) AST_SCHEMA_VERSION.invokeExact();
        } catch (Throwable t) {
            throw wrap(t);
        }
    }

    static RuntimeException wrap(Throwable t) {
        if (t instanceof RuntimeException runtime) {
            return runtime;
        }
        if (t instanceof Error error) {
            throw error;
        }
        return new MarkstoneException("markstone: native invocation failed", t);
    }
}
