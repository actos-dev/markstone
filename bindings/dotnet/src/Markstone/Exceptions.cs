namespace Markstone;

/// <summary>
/// Base exception thrown for errors originating from the markstone engine.
/// </summary>
public class MarkstoneException : Exception
{
    /// <summary>
    /// Initializes a new instance of <see cref="MarkstoneException"/> with a message.
    /// </summary>
    /// <param name="message">Error description.</param>
    public MarkstoneException(string message) : base(message) { }

    /// <summary>
    /// Initializes a new instance of <see cref="MarkstoneException"/> with a message and inner exception.
    /// </summary>
    /// <param name="message">Error description.</param>
    /// <param name="innerException">Inner cause.</param>
    public MarkstoneException(string message, Exception innerException) : base(message, innerException) { }
}

/// <summary>
/// Thrown when the markdown input exceeds the 4 MiB maximum limit.
/// </summary>
public class InputTooLargeException : MarkstoneException
{
    /// <summary>
    /// Initializes a new instance of <see cref="InputTooLargeException"/> with a message.
    /// </summary>
    /// <param name="message">Error description.</param>
    public InputTooLargeException(string message) : base(message) { }

    /// <summary>
    /// Initializes a new instance of <see cref="InputTooLargeException"/> with a message and inner exception.
    /// </summary>
    /// <param name="message">Error description.</param>
    /// <param name="innerException">Inner cause.</param>
    public InputTooLargeException(string message, Exception innerException) : base(message, innerException) { }
}

/// <summary>
/// Thrown when the markdown block nesting depth exceeds the maximum limit of 64.
/// </summary>
public class DepthExceededException : MarkstoneException
{
    /// <summary>
    /// Initializes a new instance of <see cref="DepthExceededException"/> with a message.
    /// </summary>
    /// <param name="message">Error description.</param>
    public DepthExceededException(string message) : base(message) { }

    /// <summary>
    /// Initializes a new instance of <see cref="DepthExceededException"/> with a message and inner exception.
    /// </summary>
    /// <param name="message">Error description.</param>
    /// <param name="innerException">Inner cause.</param>
    public DepthExceededException(string message, Exception innerException) : base(message, innerException) { }
}

/// <summary>
/// Thrown when the markdown input contains invalid UTF-8 byte sequences.
/// </summary>
public class InvalidUtf8Exception : MarkstoneException
{
    /// <summary>
    /// Initializes a new instance of <see cref="InvalidUtf8Exception"/> with a message.
    /// </summary>
    /// <param name="message">Error description.</param>
    public InvalidUtf8Exception(string message) : base(message) { }

    /// <summary>
    /// Initializes a new instance of <see cref="InvalidUtf8Exception"/> with a message and inner exception.
    /// </summary>
    /// <param name="message">Error description.</param>
    /// <param name="innerException">Inner cause.</param>
    public InvalidUtf8Exception(string message, Exception innerException) : base(message, innerException) { }
}

internal static class Exceptions
{
    internal const int StatusOk = 0;
    internal const int StatusErrNullArgument = 1;
    internal const int StatusErrInvalidUtf8 = 2;
    internal const int StatusErrInputTooLarge = 3;
    internal const int StatusErrDepthExceeded = 4;
    internal const int StatusErrInternal = 5;

    internal static void ThrowIfError(int status)
    {
        switch (status)
        {
            case StatusOk:
                return;
            case StatusErrNullArgument:
                throw new ArgumentException("markstone: null argument passed to native function.");
            case StatusErrInvalidUtf8:
                throw new InvalidUtf8Exception("markstone: input contains invalid UTF-8 byte sequences.");
            case StatusErrInputTooLarge:
                throw new InputTooLargeException("markstone: markdown input exceeds 4 MiB limit.");
            case StatusErrDepthExceeded:
                throw new DepthExceededException("markstone: markdown block nesting depth exceeds limit of 64.");
            case StatusErrInternal:
                throw new MarkstoneException("markstone: internal error or caught panic.");
            default:
                throw new MarkstoneException($"markstone: unknown native error status {status}.");
        }
    }
}
