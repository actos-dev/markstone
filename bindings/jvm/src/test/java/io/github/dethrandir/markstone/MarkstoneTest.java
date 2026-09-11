package io.github.dethrandir.markstone;

import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.Callable;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

public class MarkstoneTest {

    @Test
    void testVersion() {
        assertEquals("0.1.0", Markstone.version());
        assertEquals("0.1.0", Markstone.Actos.version());
    }

    @Test
    void testAstSchemaVersion() {
        assertEquals(1, Markstone.AST_SCHEMA_VERSION);
        assertEquals(1, Markstone.Actos.AST_SCHEMA_VERSION);
    }

    @Test
    void testNullInput() {
        assertThrows(NullPointerException.class, () -> Markstone.toHtml(null));
        assertThrows(NullPointerException.class, () -> Markstone.toAst(null));
        assertThrows(NullPointerException.class, () -> Markstone.Actos.toHtml(null));
        assertThrows(NullPointerException.class, () -> Markstone.Actos.toAst(null));
    }

    @Test
    void testEmptyInput() {
        assertEquals("", Markstone.toHtml(""));
        assertEquals("", Markstone.Actos.toHtml(""));

        String ast = Markstone.toAst("");
        assertNotNull(ast);
        assertTrue(ast.contains("\"schema\":1"));
        assertTrue(ast.contains("\"type\":\"document\""));

        String actosAst = Markstone.Actos.toAst("");
        assertNotNull(actosAst);
        assertTrue(actosAst.contains("\"schema\":1"));
        assertTrue(actosAst.contains("\"type\":\"document\""));
    }

    @Test
    void testInputTooLarge() {
        // 4 MiB limit: 4 * 1024 * 1024 + 1
        String tooLarge = "a".repeat(4 * 1024 * 1024 + 1);

        assertThrows(InputTooLargeException.class, () -> Markstone.toHtml(tooLarge));
        assertThrows(InputTooLargeException.class, () -> Markstone.toAst(tooLarge));
        assertThrows(InputTooLargeException.class, () -> Markstone.Actos.toHtml(tooLarge));
        assertThrows(InputTooLargeException.class, () -> Markstone.Actos.toAst(tooLarge));
    }

    @Test
    void testDepthExceeded() {
        // 64 block nesting depth limit: 64 blockquotes + 1 paragraph = depth 65 > 64
        String tooDeep = "> ".repeat(64) + "deep content\n";

        assertThrows(DepthExceededException.class, () -> Markstone.toHtml(tooDeep));
        assertThrows(DepthExceededException.class, () -> Markstone.toAst(tooDeep));
        assertThrows(DepthExceededException.class, () -> Markstone.Actos.toHtml(tooDeep));
        assertThrows(DepthExceededException.class, () -> Markstone.Actos.toAst(tooDeep));
    }

    @Test
    void testDepthWithinLimit() {
        // 63 nested blockquotes + 1 paragraph = depth 64 <= 64
        String validDepth = "> ".repeat(63) + "valid content\n";
        String html = Markstone.toHtml(validDepth);
        assertNotNull(html);
        assertTrue(html.contains("valid content"));
    }

    @Test
    void testExceptionHierarchy() {
        assertTrue(MarkstoneException.class.isAssignableFrom(InputTooLargeException.class));
        assertTrue(MarkstoneException.class.isAssignableFrom(DepthExceededException.class));
        assertTrue(MarkstoneException.class.isAssignableFrom(InvalidUtf8Exception.class));
        assertTrue(RuntimeException.class.isAssignableFrom(MarkstoneException.class));
    }

    @Test
    void testUnicodeAndSpecialCharacters() {
        String input = "# 🌍 Unicode & 1 < 2 & 3 > 2 \n\n日本語 text, العربية, and `<code>`.\n";
        String html = Markstone.toHtml(input);
        assertTrue(html.contains("<h1>🌍 Unicode &amp; 1 &lt; 2 &amp; 3 &gt; 2</h1>"));
        assertTrue(html.contains("日本語"));
        assertTrue(html.contains("العربية"));
        assertTrue(html.contains("<code>&lt;code&gt;</code>"));

        String ast = Markstone.toAst(input);
        assertTrue(ast.contains("🌍"));
        assertTrue(ast.contains("日本語"));
        assertTrue(ast.contains("العربية"));
    }

    @Test
    void testActosMentionsAndTags() {
        String input = "Hello @alice and @bob_123, check #rust-lang and #v1!";
        String html = Markstone.Actos.toHtml(input);
        assertTrue(html.contains("<a href=\"/u/alice\" class=\"mention\">@alice</a>"));
        assertTrue(html.contains("<a href=\"/u/bob_123\" class=\"mention\">@bob_123</a>"));
        assertTrue(html.contains("<a href=\"/t/rust-lang\" class=\"tag\">#rust-lang</a>"));
        assertTrue(html.contains("<a href=\"/t/v1\" class=\"tag\">#v1</a>"));

        // Generic should NOT link mentions or tags
        String genericHtml = Markstone.toHtml(input);
        assertTrue(genericHtml.contains("@alice"));
        assertTrue(!genericHtml.contains("<a href=\"/u/alice\""));
    }

    @Test
    void testThreadSafety() throws Exception {
        int threads = 8;
        int iterations = 100;
        ExecutorService executor = Executors.newFixedThreadPool(threads);
        List<Callable<Void>> tasks = new ArrayList<>();

        for (int i = 0; i < threads; i++) {
            tasks.add(() -> {
                for (int j = 0; j < iterations; j++) {
                    String input = "## Heading " + j + "\n\nParagraph text with @user and #tag.";
                    String html = Markstone.toHtml(input);
                    assertTrue(html.startsWith("<h2>Heading "));

                    String actosHtml = Markstone.Actos.toHtml(input);
                    assertTrue(actosHtml.contains("class=\"mention\""));

                    String ast = Markstone.toAst(input);
                    assertTrue(ast.contains("\"schema\":1"));

                    String actosAst = Markstone.Actos.toAst(input);
                    assertTrue(actosAst.contains("\"type\":\"mention\""));
                }
                return null;
            });
        }

        List<Future<Void>> futures = executor.invokeAll(tasks);
        for (Future<Void> future : futures) {
            future.get();
        }
        executor.shutdown();
    }

    @Test
    void testMemoryRepeatedAllocations() {
        for (int i = 0; i < 5000; i++) {
            String html = Markstone.toHtml("Short markdown test " + i);
            assertEquals("<p>Short markdown test " + i + "</p>\n", html);
        }
    }
}
