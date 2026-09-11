package io.github.dethrandir.markstone

/**
 * AST JSON schema version. Increments when the schema breaks.
 */
const val AST_SCHEMA_VERSION: Int = Markstone.AST_SCHEMA_VERSION

/**
 * Returns the markstone semver version string.
 */
fun version(): String = Markstone.version()

/**
 * Parses generic CommonMark + GFM markdown and renders safe HTML.
 */
fun toHtml(input: String): String = Markstone.toHtml(input)

/**
 * Parses generic CommonMark + GFM markdown and produces AST JSON.
 */
fun toAst(input: String): String = Markstone.toAst(input)

/**
 * Actos family: generic CommonMark + GFM pipeline plus Actos-specific passes
 * (@mention and #tag linking).
 */
object Actos {
    const val AST_SCHEMA_VERSION: Int = Markstone.AST_SCHEMA_VERSION

    fun version(): String = Markstone.version()

    fun toHtml(input: String): String = Markstone.Actos.toHtml(input)

    fun toAst(input: String): String = Markstone.Actos.toAst(input)
}

/**
 * Extension function on String to render generic CommonMark + GFM HTML.
 */
fun String.toMarkdownHtml(): String = Markstone.toHtml(this)

/**
 * Extension function on String to produce generic CommonMark + GFM AST JSON.
 */
fun String.toMarkdownAst(): String = Markstone.toAst(this)

/**
 * Extension function on String to render Actos markdown HTML (@mention and #tag linking).
 */
fun String.toActosMarkdownHtml(): String = Markstone.Actos.toHtml(this)

/**
 * Extension function on String to produce Actos markdown AST JSON (@mention and #tag nodes).
 */
fun String.toActosMarkdownAst(): String = Markstone.Actos.toAst(this)
