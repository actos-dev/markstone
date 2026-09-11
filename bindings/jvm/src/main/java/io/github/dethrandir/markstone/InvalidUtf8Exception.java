package io.github.dethrandir.markstone;

/**
 * Thrown when the markdown input contains invalid UTF-8 byte sequences.
 */
public class InvalidUtf8Exception extends MarkstoneException {

    private static final long serialVersionUID = 1L;

    public InvalidUtf8Exception(String message) {
        super(message);
    }

    public InvalidUtf8Exception(String message, Throwable cause) {
        super(message, cause);
    }
}
