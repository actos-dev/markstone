package io.github.dethrandir.markstone;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * CLI runner entry point for the markstone conformance test harness.
 */
public final class ConformanceRunner {

    private ConformanceRunner() {
    }

    public static void main(String[] args) {
        String mode = null;
        String inputPath = null;

        for (int i = 0; i < args.length; i++) {
            if ("--mode".equals(args[i]) && i + 1 < args.length) {
                mode = args[++i];
            } else if (!args[i].startsWith("-") && inputPath == null) {
                inputPath = args[i];
            }
        }

        if (mode == null) {
            System.err.println("Error: --mode is required");
            System.exit(1);
        }

        String input;
        try {
            if (inputPath != null && !"-".equals(inputPath)) {
                input = Files.readString(Path.of(inputPath), StandardCharsets.UTF_8);
            } else {
                byte[] inBytes = System.in.readAllBytes();
                input = new String(inBytes, StandardCharsets.UTF_8);
            }
        } catch (IOException e) {
            System.err.println("Error reading input: " + e.getMessage());
            System.exit(1);
            return;
        }

        String output;
        switch (mode) {
            case "generic-html":
                output = Markstone.toHtml(input);
                break;
            case "generic-ast":
                output = Markstone.toAst(input);
                break;
            case "actos-html":
                output = Markstone.Actos.toHtml(input);
                break;
            case "actos-ast":
                output = Markstone.Actos.toAst(input);
                break;
            default:
                System.err.println("Unknown mode: " + mode);
                System.exit(1);
                return;
        }

        byte[] outBytes = output.getBytes(StandardCharsets.UTF_8);
        try {
            System.out.write(outBytes);
            System.out.flush();
        } catch (IOException e) {
            System.err.println("Error writing output: " + e.getMessage());
            System.exit(1);
        }
    }
}
