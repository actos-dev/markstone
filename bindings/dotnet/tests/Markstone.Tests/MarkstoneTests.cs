using System.Text;
using Xunit;

namespace Markstone.Tests;

/// <summary>
/// Unit tests for limits, exceptions, thread safety, memory stability, and Actos features.
/// </summary>
public class MarkstoneTests
{
    [Fact]
    public void TestVersion()
    {
        Assert.Equal("0.1.0", Markstone.Version);
        Assert.Equal("0.1.0", Markstone.Actos.Version);
    }

    [Fact]
    public void TestAstSchemaVersion()
    {
        Assert.Equal(1, Markstone.AstSchemaVersion);
        Assert.Equal(1, Markstone.Actos.AstSchemaVersion);
        Assert.Equal((uint)1, NativeMethods.markstone_ast_schema_version());
    }

    [Fact]
    public void TestNullInputThrowsArgumentNullException()
    {
        Assert.Throws<ArgumentNullException>(() => Markstone.ToHtml((string)null!));
        Assert.Throws<ArgumentNullException>(() => Markstone.ToAst((string)null!));
        Assert.Throws<ArgumentNullException>(() => Markstone.Actos.ToHtml((string)null!));
        Assert.Throws<ArgumentNullException>(() => Markstone.Actos.ToAst((string)null!));
    }

    [Fact]
    public void TestEmptyInputProducesValidOutput()
    {
        Assert.Equal(string.Empty, Markstone.ToHtml(string.Empty));
        Assert.Equal(string.Empty, Markstone.Actos.ToHtml(string.Empty));
        Assert.Equal(string.Empty, Markstone.ToHtml(ReadOnlySpan<byte>.Empty));
        Assert.Equal(string.Empty, Markstone.Actos.ToHtml(ReadOnlySpan<byte>.Empty));

        var ast = Markstone.ToAst(string.Empty);
        Assert.NotNull(ast);
        Assert.Contains("\"schema\":1", ast);
        Assert.Contains("\"type\":\"document\"", ast);

        var astSpan = Markstone.ToAst(ReadOnlySpan<byte>.Empty);
        Assert.Equal(ast, astSpan);

        var actosAst = Markstone.Actos.ToAst(string.Empty);
        Assert.NotNull(actosAst);
        Assert.Contains("\"schema\":1", actosAst);
        Assert.Contains("\"type\":\"document\"", actosAst);

        var actosAstSpan = Markstone.Actos.ToAst(ReadOnlySpan<byte>.Empty);
        Assert.Equal(actosAst, actosAstSpan);
    }

    [Fact]
    public void TestInputTooLargeThrowsInputTooLargeException()
    {
        // Limit is 4 MiB (4 * 1024 * 1024 = 4194304 bytes). Exceed by 1 byte.
        var tooLargeString = new string('a', (4 * 1024 * 1024) + 1);
        var tooLargeBytes = new byte[(4 * 1024 * 1024) + 1];
        Array.Fill(tooLargeBytes, (byte)'a');

        // String overloads
        Assert.Throws<InputTooLargeException>(() => Markstone.ToHtml(tooLargeString));
        Assert.Throws<InputTooLargeException>(() => Markstone.ToAst(tooLargeString));
        Assert.Throws<InputTooLargeException>(() => Markstone.Actos.ToHtml(tooLargeString));
        Assert.Throws<InputTooLargeException>(() => Markstone.Actos.ToAst(tooLargeString));

        // ReadOnlySpan<byte> overloads
        Assert.Throws<InputTooLargeException>(() => Markstone.ToHtml(tooLargeBytes));
        Assert.Throws<InputTooLargeException>(() => Markstone.ToAst(tooLargeBytes));
        Assert.Throws<InputTooLargeException>(() => Markstone.Actos.ToHtml(tooLargeBytes));
        Assert.Throws<InputTooLargeException>(() => Markstone.Actos.ToAst(tooLargeBytes));
    }

    [Fact]
    public void TestDepthExceededThrowsDepthExceededException()
    {
        // 64 nested blockquotes + 1 paragraph = depth 65 > 64 limit
        var tooDeepString = string.Concat(Enumerable.Repeat("> ", 64)) + "deep content\n";
        var tooDeepBytes = Encoding.UTF8.GetBytes(tooDeepString);

        // String overloads
        Assert.Throws<DepthExceededException>(() => Markstone.ToHtml(tooDeepString));
        Assert.Throws<DepthExceededException>(() => Markstone.ToAst(tooDeepString));
        Assert.Throws<DepthExceededException>(() => Markstone.Actos.ToHtml(tooDeepString));
        Assert.Throws<DepthExceededException>(() => Markstone.Actos.ToAst(tooDeepString));

        // ReadOnlySpan<byte> overloads
        Assert.Throws<DepthExceededException>(() => Markstone.ToHtml(tooDeepBytes));
        Assert.Throws<DepthExceededException>(() => Markstone.ToAst(tooDeepBytes));
        Assert.Throws<DepthExceededException>(() => Markstone.Actos.ToHtml(tooDeepBytes));
        Assert.Throws<DepthExceededException>(() => Markstone.Actos.ToAst(tooDeepBytes));
    }

    [Fact]
    public void TestDepthWithinLimitSucceeds()
    {
        // 63 nested blockquotes + 1 paragraph = depth 64 <= 64 limit
        var validDepth = string.Concat(Enumerable.Repeat("> ", 63)) + "valid content\n";
        var html = Markstone.ToHtml(validDepth);
        Assert.NotNull(html);
        Assert.Contains("valid content", html);
    }

    [Fact]
    public void TestInvalidUtf8ThrowsInvalidUtf8Exception()
    {
        byte[] invalidUtf8 = [0xFF, 0xFE, 0xFD];

        Assert.Throws<InvalidUtf8Exception>(() => Markstone.ToHtml(invalidUtf8));
        Assert.Throws<InvalidUtf8Exception>(() => Markstone.ToAst(invalidUtf8));
        Assert.Throws<InvalidUtf8Exception>(() => Markstone.Actos.ToHtml(invalidUtf8));
        Assert.Throws<InvalidUtf8Exception>(() => Markstone.Actos.ToAst(invalidUtf8));
    }

    [Fact]
    public void TestExceptionHierarchy()
    {
        Assert.True(typeof(MarkstoneException).IsAssignableFrom(typeof(InputTooLargeException)));
        Assert.True(typeof(MarkstoneException).IsAssignableFrom(typeof(DepthExceededException)));
        Assert.True(typeof(MarkstoneException).IsAssignableFrom(typeof(InvalidUtf8Exception)));
        Assert.True(typeof(Exception).IsAssignableFrom(typeof(MarkstoneException)));
    }

    [Fact]
    public void TestUnicodeAndSpecialCharacters()
    {
        var input = "# 🌍 Unicode & 1 < 2 & 3 > 2 \n\n日本語 text, العربية, and `<code>`.\n";
        var html = Markstone.ToHtml(input);
        Assert.Contains("<h1>🌍 Unicode &amp; 1 &lt; 2 &amp; 3 &gt; 2</h1>", html);
        Assert.Contains("日本語", html);
        Assert.Contains("العربية", html);
        Assert.Contains("<code>&lt;code&gt;</code>", html);

        var ast = Markstone.ToAst(input);
        Assert.Contains("🌍", ast);
        Assert.Contains("日本語", ast);
        Assert.Contains("العربية", ast);
    }

    [Fact]
    public void TestActosMentionsAndTags()
    {
        var input = "Hello @alice and @bob_123, check #rust-lang and #v1!";

        var actosHtml = Markstone.Actos.ToHtml(input);
        Assert.Contains("<a href=\"/u/alice\" class=\"mention\">@alice</a>", actosHtml);
        Assert.Contains("<a href=\"/u/bob_123\" class=\"mention\">@bob_123</a>", actosHtml);
        Assert.Contains("<a href=\"/t/rust-lang\" class=\"tag\">#rust-lang</a>", actosHtml);
        Assert.Contains("<a href=\"/t/v1\" class=\"tag\">#v1</a>", actosHtml);

        var genericHtml = Markstone.ToHtml(input);
        Assert.Contains("@alice", genericHtml);
        Assert.DoesNotContain("<a href=\"/u/alice\"", genericHtml);
        Assert.DoesNotContain("<a href=\"/t/rust-lang\"", genericHtml);
    }

    [Fact]
    public async Task TestThreadSafety()
    {
        const int threads = 8;
        const int iterations = 100;
        var tasks = new Task[threads];

        for (int i = 0; i < threads; i++)
        {
            tasks[i] = Task.Run(() =>
            {
                for (int j = 0; j < iterations; j++)
                {
                    var input = $"## Heading {j}\n\nParagraph text with @user and #tag.";

                    var html = Markstone.ToHtml(input);
                    Assert.StartsWith("<h2>Heading ", html);

                    var actosHtml = Markstone.Actos.ToHtml(input);
                    Assert.Contains("class=\"mention\"", actosHtml);

                    var ast = Markstone.ToAst(input);
                    Assert.Contains("\"schema\":1", ast);

                    var actosAst = Markstone.Actos.ToAst(input);
                    Assert.Contains("\"type\":\"mention\"", actosAst);
                }
            });
        }

        await Task.WhenAll(tasks);
    }

    [Fact]
    public void TestMemoryRepeatedAllocations()
    {
        for (int i = 0; i < 5000; i++)
        {
            var html = Markstone.ToHtml($"Short markdown test {i}");
            Assert.Equal($"<p>Short markdown test {i}</p>\n", html);
        }
    }
}
