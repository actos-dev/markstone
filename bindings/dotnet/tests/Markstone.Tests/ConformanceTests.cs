using System.Text;
using Xunit;

namespace Markstone.Tests;

/// <summary>
/// Conformance suite testing all 77 cases (308 checks) against golden files.
/// Asserts byte-for-byte identity for both string and ReadOnlySpan&lt;byte&gt; APIs.
/// </summary>
public class ConformanceTests
{
    private static string FindCasesDir()
    {
        var dir = AppContext.BaseDirectory;
        for (var i = 0; i < 10 && dir is not null; i++)
        {
            var candidate = Path.Combine(dir, "conformance", "cases");
            if (Directory.Exists(candidate))
            {
                return candidate;
            }
            dir = Path.GetDirectoryName(dir.TrimEnd(Path.DirectorySeparatorChar));
        }

        var cwd = Directory.GetCurrentDirectory();
        for (var i = 0; i < 10 && cwd is not null; i++)
        {
            var candidate = Path.Combine(cwd, "conformance", "cases");
            if (Directory.Exists(candidate))
            {
                return candidate;
            }
            cwd = Path.GetDirectoryName(cwd.TrimEnd(Path.DirectorySeparatorChar));
        }

        throw new DirectoryNotFoundException("Could not locate conformance/cases directory");
    }

    public static IEnumerable<object[]> GetCases()
    {
        var casesDir = FindCasesDir();
        var dirs = Directory.GetDirectories(casesDir)
            .Where(d => File.Exists(Path.Combine(d, "input.md")))
            .OrderBy(Path.GetFileName, StringComparer.Ordinal)
            .ToList();

        foreach (var d in dirs)
        {
            yield return new object[] { Path.GetFileName(d), d };
        }
    }

    [Fact]
    public void ConformanceCaseCountIs77()
    {
        var casesDir = FindCasesDir();
        var count = Directory.GetDirectories(casesDir)
            .Count(d => File.Exists(Path.Combine(d, "input.md")));
        Assert.Equal(77, count);
    }

    [Theory]
    [MemberData(nameof(GetCases))]
    public void ConformanceCaseMatchesGoldens(string caseName, string caseDir)
    {
        Assert.False(string.IsNullOrEmpty(caseName));
        var input = File.ReadAllText(Path.Combine(caseDir, "input.md"), Encoding.UTF8);
        var inputBytes = File.ReadAllBytes(Path.Combine(caseDir, "input.md"));

        // 1. generic-html
        var expectedGenericHtml = File.ReadAllBytes(Path.Combine(caseDir, "generic.html"));
        var actualGenericHtml = Encoding.UTF8.GetBytes(Markstone.ToHtml(input));
        var actualGenericHtmlSpan = Encoding.UTF8.GetBytes(Markstone.ToHtml(inputBytes));
        Assert.Equal(expectedGenericHtml, actualGenericHtml);
        Assert.Equal(expectedGenericHtml, actualGenericHtmlSpan);

        // 2. generic-ast
        var expectedGenericAst = File.ReadAllBytes(Path.Combine(caseDir, "generic.ast.json"));
        var actualGenericAst = Encoding.UTF8.GetBytes(Markstone.ToAst(input));
        var actualGenericAstSpan = Encoding.UTF8.GetBytes(Markstone.ToAst(inputBytes));
        Assert.Equal(expectedGenericAst, actualGenericAst);
        Assert.Equal(expectedGenericAst, actualGenericAstSpan);

        // 3. actos-html
        var expectedActosHtml = File.ReadAllBytes(Path.Combine(caseDir, "actos.html"));
        var actualActosHtml = Encoding.UTF8.GetBytes(Markstone.Actos.ToHtml(input));
        var actualActosHtmlSpan = Encoding.UTF8.GetBytes(Markstone.Actos.ToHtml(inputBytes));
        Assert.Equal(expectedActosHtml, actualActosHtml);
        Assert.Equal(expectedActosHtml, actualActosHtmlSpan);

        // 4. actos-ast
        var expectedActosAst = File.ReadAllBytes(Path.Combine(caseDir, "actos.ast.json"));
        var actualActosAst = Encoding.UTF8.GetBytes(Markstone.Actos.ToAst(input));
        var actualActosAstSpan = Encoding.UTF8.GetBytes(Markstone.Actos.ToAst(inputBytes));
        Assert.Equal(expectedActosAst, actualActosAst);
        Assert.Equal(expectedActosAst, actualActosAstSpan);
    }
}
