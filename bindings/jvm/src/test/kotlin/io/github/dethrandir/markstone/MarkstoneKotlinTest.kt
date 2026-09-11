package io.github.dethrandir.markstone

import org.junit.jupiter.api.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

class MarkstoneKotlinTest {

    @Test
    fun testTopLevelFunctions() {
        val markdown = "# Hello Kotlin"
        val html = toHtml(markdown)
        assertEquals("<h1>Hello Kotlin</h1>\n", html)

        val ast = toAst(markdown)
        assertTrue(ast.contains("\"schema\":1"))
        assertTrue(ast.contains("Hello Kotlin"))

        assertEquals("0.1.0", version())
        assertEquals(1, AST_SCHEMA_VERSION)
    }

    @Test
    fun testActosObject() {
        val markdown = "Hello @kotlin and #jvm"
        val html = Actos.toHtml(markdown)
        assertTrue(html.contains("<a href=\"/u/kotlin\" class=\"mention\">@kotlin</a>"))
        assertTrue(html.contains("<a href=\"/t/jvm\" class=\"tag\">#jvm</a>"))

        val ast = Actos.toAst(markdown)
        assertTrue(ast.contains("\"type\":\"mention\""))
        assertTrue(ast.contains("\"username\":\"kotlin\""))
        assertTrue(ast.contains("\"type\":\"tag\""))

        assertEquals("0.1.0", Actos.version())
        assertEquals(1, Actos.AST_SCHEMA_VERSION)
    }

    @Test
    fun testExtensionFunctions() {
        val text = "## Section\n\nDiscussion with @alice about #testing."

        val genericHtml = text.toMarkdownHtml()
        assertTrue(genericHtml.startsWith("<h2>Section</h2>"))
        assertTrue(genericHtml.contains("@alice"))
        assertTrue(!genericHtml.contains("<a href=\"/u/alice\""))

        val genericAst = text.toMarkdownAst()
        assertTrue(genericAst.contains("\"type\":\"heading\""))
        assertTrue(!genericAst.contains("\"type\":\"mention\""))

        val actosHtml = text.toActosMarkdownHtml()
        assertTrue(actosHtml.contains("<a href=\"/u/alice\" class=\"mention\">@alice</a>"))
        assertTrue(actosHtml.contains("<a href=\"/t/testing\" class=\"tag\">#testing</a>"))

        val actosAst = text.toActosMarkdownAst()
        assertTrue(actosAst.contains("\"type\":\"mention\""))
        assertTrue(actosAst.contains("\"username\":\"alice\""))
        assertTrue(actosAst.contains("\"type\":\"tag\""))
        assertTrue(actosAst.contains("\"name\":\"testing\""))
    }

    @Test
    fun testExceptionsInKotlin() {
        val tooLarge = "k".repeat(4 * 1024 * 1024 + 1)
        assertFailsWith<InputTooLargeException> {
            toHtml(tooLarge)
        }
        assertFailsWith<InputTooLargeException> {
            tooLarge.toMarkdownHtml()
        }

        val tooDeep = "> ".repeat(65) + "nested"
        assertFailsWith<DepthExceededException> {
            toAst(tooDeep)
        }
        assertFailsWith<DepthExceededException> {
            tooDeep.toActosMarkdownAst()
        }
    }
}
