package io.github.dethrandir.markstone;

/**
 * Base unchecked exception thrown when a markstone native operation fails.
 */
public class MarkstoneException extends RuntimeException {

    private static final long serialVersionUID = 1L;

    public MarkstoneException(String message) {
        super(message);
    }

    public MarkstoneException(String message, Throwable cause) {
        super(message, cause);
    }
}
