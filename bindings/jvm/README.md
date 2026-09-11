# markstone (JVM)

Java and Kotlin bindings for markstone: fast, safe Markdown-to-HTML and AST engine with Actos extensions, powered by the Java Foreign Function & Memory API (FFM).

Requires JDK 22+.

## Maven Dependency

```xml
<dependency>
  <groupId>io.github.dethrandir</groupId>
  <artifactId>markstone</artifactId>
  <version>0.1.0</version>
</dependency>
```

> **Note:** Because FFM invokes native functions, your application command line must include:
> ```bash
> --enable-native-access=ALL-UNNAMED
> ```
> or specify `<Enable-Native-Access>ALL-UNNAMED</Enable-Native-Access>` in your JAR manifest.

## Usage in Java

```java
import io.github.dethrandir.markstone.Markstone;

// Generic Markdown (CommonMark 0.31 + GFM)
String html = Markstone.toHtml("# Hello World");
String ast = Markstone.toAst("# Hello World");

// Actos extensions (@mention and #tag linking)
String actosHtml = Markstone.Actos.toHtml("Hello @alice and #rust");
String actosAst = Markstone.Actos.toAst("Hello @alice and #rust");

// Metadata
String version = Markstone.version();
int schemaVersion = Markstone.AST_SCHEMA_VERSION;
```

## Usage in Kotlin

```kotlin
import io.github.dethrandir.markstone.*

// Top-level functions
val html = toHtml("# Hello World")
val ast = toAst("# Hello World")

// Actos object
val actosHtml = Actos.toHtml("Hello @alice and #rust")
val actosAst = Actos.toAst("Hello @alice and #rust")

// Idiomatic extension functions on String
val extHtml = "# Hello World".toMarkdownHtml()
val extAst = "# Hello World".toMarkdownAst()
val extActosHtml = "Hello @alice and #rust".toActosMarkdownHtml()
val extActosAst = "Hello @alice and #rust".toActosMarkdownAst()

// Metadata
val version = version()
val schemaVersion = AST_SCHEMA_VERSION
```

## Exception Hierarchy

All exceptions thrown by markstone are unchecked runtime exceptions:

- `MarkstoneException` (base class)
  - `InputTooLargeException` (exceeds 4 MiB)
  - `DepthExceededException` (block nesting depth exceeds 64)
  - `InvalidUtf8Exception` (invalid UTF-8 bytes)

## Native Library Loading

The native library is resolved in the following priority:

1. `MARKSTONE_LIBRARY_PATH` environment variable (points to library file or directory).
2. Local build directory (`target/release` or `target/debug`).
3. Classpath resource bundled inside the JAR (`native/{os}-{arch}/libmarkstone_abi.so`).
4. System library path fallback via `System.loadLibrary("markstone_abi")`.
