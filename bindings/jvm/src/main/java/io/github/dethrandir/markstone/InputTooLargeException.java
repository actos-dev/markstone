package io.github.dethrandir.markstone;

/**
 * Thrown when the markdown input exceeds the maximum permitted input size (4 MiB).
 */
public class InputTooLargeException extends MarkstoneException {

    private static final long serialVersionUID = 1L;

    public InputTooLargeException(String message) {
        super(message);
    }

    public InputTooLargeException(String message, Throwable cause) {
        super(message, cause);
    }
}
