# markstone (.NET)

Official .NET C# bindings for markstone: fast, safe Markdown-to-HTML and AST engine with Actos extensions, powered by native P/Invoke.

Targets .NET 8.0+ (`net8.0`). Fully AOT-compatible and trimming-friendly.

## Installation

Add the package via the .NET CLI:

```bash
dotnet add package Markstone
```

Or reference it in your `.csproj`:

```xml
<PackageReference Include="Markstone" Version="0.1.0" />
```

Prebuilt native binaries are distributed within the NuGet package under `runtimes/{rid}/native/`.

## Usage in C#

```csharp
using Markstone;

// Generic Markdown (CommonMark 0.31 + GFM)
string html = Markstone.ToHtml("# Hello World\n\nVisit https://example.com");
string ast = Markstone.ToAst("# Hello World");

// Zero-allocation byte span overload
ReadOnlySpan<byte> utf8Bytes = "# Hello"u8;
string spanHtml = Markstone.ToHtml(utf8Bytes);

// Actos extensions (@mention and #tag linking)
string actosHtml = Markstone.Actos.ToHtml("Hello @alice and #rust-lang!");
string actosAst = Markstone.Actos.ToAst("Hello @alice and #rust-lang!");

// Metadata
string version = Markstone.Version; // "0.1.0"
int schemaVersion = Markstone.AstSchemaVersion; // 1
```

## Exception Hierarchy

All exceptions thrown by markstone inherit from `MarkstoneException`:

- `MarkstoneException`
  - `InputTooLargeException` — markdown input exceeds 4 MiB
  - `DepthExceededException` — block nesting depth exceeds 64
  - `InvalidUtf8Exception` — input contains invalid UTF-8 byte sequences

When `null` strings are passed, standard `ArgumentNullException` is thrown.

## Native Library Resolution

The library automatically resolves the native dynamic library in the following order:

1. Standard NuGet runtime layout: `runtimes/{rid}/native/`.
2. Environment variables: `MARKSTONE_LIBRARY_PATH` (file or directory) or `MARKSTONE_NATIVE_DIR` (directory).
3. Walking up the directory hierarchy to probe local repository build output (`target/release/`, `target/debug/`).

## License

MIT License.
