package io.github.dethrandir.markstone;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.MethodSource;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Comparator;
import java.util.List;
import java.util.stream.Stream;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ConformanceTest {

    private static Path findCasesDir() {
        Path dir = Path.of("").toAbsolutePath();
        for (int i = 0; i < 8 && dir != null; i++) {
            Path cases = dir.resolve("conformance/cases");
            if (Files.isDirectory(cases)) {
                return cases;
            }
            dir = dir.getParent();
        }
        throw new IllegalStateException("Could not find conformance/cases directory");
    }

    static Stream<Path> provideCaseDirectories() throws IOException {
        Path casesDir = findCasesDir();
        try (Stream<Path> stream = Files.list(casesDir)) {
            List<Path> list = stream
                    .filter(Files::isDirectory)
                    .filter(p -> Files.isRegularFile(p.resolve("input.md")))
                    .sorted(Comparator.comparing(Path::getFileName))
                    .toList();
            assertEquals(77, list.size(), "Expected 77 conformance cases");
            return list.stream();
        }
    }

    @ParameterizedTest(name = "{0}")
    @MethodSource("provideCaseDirectories")
    @DisplayName("Conformance case byte-for-byte check")
    void testConformanceCase(Path caseDir) throws IOException {
        String input = Files.readString(caseDir.resolve("input.md"), StandardCharsets.UTF_8);

        // 1. generic-html
        byte[] expectedGenericHtml = Files.readAllBytes(caseDir.resolve("generic.html"));
        byte[] actualGenericHtml = Markstone.toHtml(input).getBytes(StandardCharsets.UTF_8);
        assertArrayEquals(expectedGenericHtml, actualGenericHtml,
                () -> "Mismatch in " + caseDir.getFileName() + " generic-html");

        // 2. generic-ast
        byte[] expectedGenericAst = Files.readAllBytes(caseDir.resolve("generic.ast.json"));
        byte[] actualGenericAst = Markstone.toAst(input).getBytes(StandardCharsets.UTF_8);
        assertArrayEquals(expectedGenericAst, actualGenericAst,
                () -> "Mismatch in " + caseDir.getFileName() + " generic-ast");

        // 3. actos-html
        byte[] expectedActosHtml = Files.readAllBytes(caseDir.resolve("actos.html"));
        byte[] actualActosHtml = Markstone.Actos.toHtml(input).getBytes(StandardCharsets.UTF_8);
        assertArrayEquals(expectedActosHtml, actualActosHtml,
                () -> "Mismatch in " + caseDir.getFileName() + " actos-html");

        // 4. actos-ast
        byte[] expectedActosAst = Files.readAllBytes(caseDir.resolve("actos.ast.json"));
        byte[] actualActosAst = Markstone.Actos.toAst(input).getBytes(StandardCharsets.UTF_8);
        assertArrayEquals(expectedActosAst, actualActosAst,
                () -> "Mismatch in " + caseDir.getFileName() + " actos-ast");
    }
}
