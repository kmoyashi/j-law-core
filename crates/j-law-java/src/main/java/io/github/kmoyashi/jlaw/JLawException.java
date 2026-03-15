package io.github.kmoyashi.jlaw;

/**
 * Exception raised when the native binding or legal calculation layer rejects an operation.
 */
public class JLawException extends RuntimeException {
    public JLawException(String message) {
        super(message);
    }

    public JLawException(String message, Throwable cause) {
        super(message, cause);
    }
}
