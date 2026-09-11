package io.github.dethrandir.markstone;

/**
 * Thrown when the markdown block nesting depth exceeds the maximum allowed limit (64).
 */
public class DepthExceededException extends MarkstoneException {

    private static final long serialVersionUID = 1L;

    public DepthExceededException(String message) {
        super(message);
    }

    public DepthExceededException(String message, Throwable cause) {
        super(message, cause);
    }
}
