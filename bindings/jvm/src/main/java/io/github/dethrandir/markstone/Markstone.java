package io.github.dethrandir.markstone;

import java.util.Objects;

/**
 * Public entry points for markdown parsing, HTML rendering, and AST generation.
 *
 * <p>Thread-safe and stateless. All operations are backed by the high-performance
 * markstone Rust core via the Java Foreign Function &amp; Memory API (FFM).
 */
public final class Markstone {

    /**
     * AST JSON schema version. Increments when the schema breaks.
     */
    public static final int AST_SCHEMA_VERSION = 1;

    private Markstone() {
    }

    /**
     * Returns the markstone semver version string (e.g. "0.1.0").
     *
     * @return version string
     */
    public static String version() {
        return Native.version();
    }

    /**
     * Parses generic CommonMark + GFM markdown and renders safe HTML.
     *
     * @param input markdown input text
     * @return safe HTML string
     * @throws NullPointerException if {@code input} is null
     * @throws InputTooLargeException if input exceeds 4 MiB
     * @throws DepthExceededException if block nesting depth exceeds 64
     * @throws InvalidUtf8Exception if input contains invalid UTF-8 bytes
     * @throws MarkstoneException on internal error
     */
    public static String toHtml(String input) {
        Objects.requireNonNull(input, "input must not be null");
        return Native.toHtml(input);
    }

    /**
     * Parses generic CommonMark + GFM markdown and produces the AST JSON representation.
     *
     * @param input markdown input text
     * @return AST JSON string
     * @throws NullPointerException if {@code input} is null
     * @throws InputTooLargeException if input exceeds 4 MiB
     * @throws DepthExceededException if block nesting depth exceeds 64
     * @throws InvalidUtf8Exception if input contains invalid UTF-8 bytes
     * @throws MarkstoneException on internal error
     */
    public static String toAst(String input) {
        Objects.requireNonNull(input, "input must not be null");
        return Native.toAst(input);
    }

    /**
     * Actos family: generic CommonMark + GFM pipeline plus Actos-specific passes
     * (e.g. &#64;mentions and #tags).
     */
    public static final class Actos {

        /**
         * AST JSON schema version for Actos AST output.
         */
        public static final int AST_SCHEMA_VERSION = 1;

        private Actos() {
        }

        /**
         * Returns the markstone semver version string.
         *
         * @return version string
         */
        public static String version() {
            return Native.version();
        }

        /**
         * Parses Actos markdown (with &#64;mention and #tag linking) and renders safe HTML.
         *
         * @param input markdown input text
         * @return safe HTML string with Actos mention and tag links
         * @throws NullPointerException if {@code input} is null
         * @throws InputTooLargeException if input exceeds 4 MiB
         * @throws DepthExceededException if block nesting depth exceeds 64
         * @throws InvalidUtf8Exception if input contains invalid UTF-8 bytes
         * @throws MarkstoneException on internal error
         */
        public static String toHtml(String input) {
            Objects.requireNonNull(input, "input must not be null");
            return Native.actosToHtml(input);
        }

        /**
         * Parses Actos markdown (with &#64;mention and #tag nodes) and produces the AST JSON.
         *
         * @param input markdown input text
         * @return AST JSON string with Actos mention and tag nodes
         * @throws NullPointerException if {@code input} is null
         * @throws InputTooLargeException if input exceeds 4 MiB
         * @throws DepthExceededException if block nesting depth exceeds 64
         * @throws InvalidUtf8Exception if input contains invalid UTF-8 bytes
         * @throws MarkstoneException on internal error
         */
        public static String toAst(String input) {
            Objects.requireNonNull(input, "input must not be null");
            return Native.actosToAst(input);
        }
    }
}
