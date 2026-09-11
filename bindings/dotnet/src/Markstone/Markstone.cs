using System.Buffers;
using System.Runtime.InteropServices;
using System.Text;

namespace Markstone;

/// <summary>
/// Public API for markdown parsing, HTML rendering, and AST JSON generation.
/// </summary>
public static class Markstone
{
    /// <summary>
    /// AST JSON schema version. Increments when the schema breaks.
    /// </summary>
    public const int AstSchemaVersion = 1;

    private static string? _version;

    /// <summary>
    /// Returns the markstone engine semver version string (e.g. "0.1.0").
    /// </summary>
    public static string Version
    {
        get
        {
            if (_version is not null)
            {
                return _version;
            }

            NativeLibraryResolver.EnsureInstalled();
            IntPtr ptr = NativeMethods.markstone_version();
            if (ptr == IntPtr.Zero)
            {
                return string.Empty;
            }

            _version = Marshal.PtrToStringUTF8(ptr) ?? string.Empty;
            return _version;
        }
    }

    private enum NativeOperation
    {
        GenericHtml,
        GenericAst,
        ActosHtml,
        ActosAst,
    }

    private static unsafe string Convert(NativeOperation op, ReadOnlySpan<byte> utf8Input)
    {
        NativeLibraryResolver.EnsureInstalled();

        IntPtr outPtr = IntPtr.Zero;
        nuint outLen = 0;
        int status;

        fixed (byte* pInput = utf8Input)
        {
            nuint len = (nuint)utf8Input.Length;
            status = op switch
            {
                NativeOperation.GenericHtml => NativeMethods.markstone_to_html(pInput, len, out outPtr, out outLen),
                NativeOperation.GenericAst => NativeMethods.markstone_to_ast(pInput, len, out outPtr, out outLen),
                NativeOperation.ActosHtml => NativeMethods.markstone_actos_to_html(pInput, len, out outPtr, out outLen),
                NativeOperation.ActosAst => NativeMethods.markstone_actos_to_ast(pInput, len, out outPtr, out outLen),
                _ => throw new ArgumentOutOfRangeException(nameof(op))
            };
        }

        Exceptions.ThrowIfError(status);

        try
        {
            if (outPtr == IntPtr.Zero || outLen == 0)
            {
                return string.Empty;
            }

            return Encoding.UTF8.GetString((byte*)outPtr, checked((int)outLen));
        }
        finally
        {
            if (outPtr != IntPtr.Zero)
            {
                NativeMethods.markstone_free(outPtr, outLen);
            }
        }
    }

    private static string ConvertString(NativeOperation op, string input)
    {
        ArgumentNullException.ThrowIfNull(input);

        if (input.Length == 0)
        {
            return Convert(op, ReadOnlySpan<byte>.Empty);
        }

        int maxByteCount = Encoding.UTF8.GetMaxByteCount(input.Length);
        if (maxByteCount <= 512)
        {
            Span<byte> buffer = stackalloc byte[512];
            int bytesWritten = Encoding.UTF8.GetBytes(input.AsSpan(), buffer);
            return Convert(op, buffer[..bytesWritten]);
        }
        else
        {
            byte[] rented = ArrayPool<byte>.Shared.Rent(maxByteCount);
            try
            {
                int bytesWritten = Encoding.UTF8.GetBytes(input.AsSpan(), rented);
                return Convert(op, rented.AsSpan(0, bytesWritten));
            }
            finally
            {
                ArrayPool<byte>.Shared.Return(rented);
            }
        }
    }

    /// <summary>
    /// Parses generic CommonMark + GFM markdown and renders safe HTML.
    /// </summary>
    /// <param name="input">Markdown input text.</param>
    /// <returns>Safe HTML output string.</returns>
    /// <exception cref="ArgumentNullException">Thrown when <paramref name="input"/> is null.</exception>
    /// <exception cref="InputTooLargeException">Thrown when input exceeds 4 MiB.</exception>
    /// <exception cref="DepthExceededException">Thrown when block nesting depth exceeds 64.</exception>
    /// <exception cref="InvalidUtf8Exception">Thrown when input contains invalid UTF-8 byte sequences.</exception>
    /// <exception cref="MarkstoneException">Thrown on internal error or panic.</exception>
    public static string ToHtml(string input) => ConvertString(NativeOperation.GenericHtml, input);

    /// <summary>
    /// Parses generic CommonMark + GFM markdown from raw UTF-8 bytes and renders safe HTML.
    /// </summary>
    /// <param name="utf8Input">UTF-8 encoded markdown input bytes.</param>
    /// <returns>Safe HTML output string.</returns>
    /// <exception cref="InputTooLargeException">Thrown when input exceeds 4 MiB.</exception>
    /// <exception cref="DepthExceededException">Thrown when block nesting depth exceeds 64.</exception>
    /// <exception cref="InvalidUtf8Exception">Thrown when input contains invalid UTF-8 byte sequences.</exception>
    /// <exception cref="MarkstoneException">Thrown on internal error or panic.</exception>
    public static string ToHtml(ReadOnlySpan<byte> utf8Input) => Convert(NativeOperation.GenericHtml, utf8Input);

    /// <summary>
    /// Parses generic CommonMark + GFM markdown and produces the AST JSON representation.
    /// </summary>
    /// <param name="input">Markdown input text.</param>
    /// <returns>AST JSON representation.</returns>
    /// <exception cref="ArgumentNullException">Thrown when <paramref name="input"/> is null.</exception>
    /// <exception cref="InputTooLargeException">Thrown when input exceeds 4 MiB.</exception>
    /// <exception cref="DepthExceededException">Thrown when block nesting depth exceeds 64.</exception>
    /// <exception cref="InvalidUtf8Exception">Thrown when input contains invalid UTF-8 byte sequences.</exception>
    /// <exception cref="MarkstoneException">Thrown on internal error or panic.</exception>
    public static string ToAst(string input) => ConvertString(NativeOperation.GenericAst, input);

    /// <summary>
    /// Parses generic CommonMark + GFM markdown from raw UTF-8 bytes and produces the AST JSON representation.
    /// </summary>
    /// <param name="utf8Input">UTF-8 encoded markdown input bytes.</param>
    /// <returns>AST JSON representation.</returns>
    /// <exception cref="InputTooLargeException">Thrown when input exceeds 4 MiB.</exception>
    /// <exception cref="DepthExceededException">Thrown when block nesting depth exceeds 64.</exception>
    /// <exception cref="InvalidUtf8Exception">Thrown when input contains invalid UTF-8 byte sequences.</exception>
    /// <exception cref="MarkstoneException">Thrown on internal error or panic.</exception>
    public static string ToAst(ReadOnlySpan<byte> utf8Input) => Convert(NativeOperation.GenericAst, utf8Input);

    /// <summary>
    /// Actos family: generic CommonMark + GFM pipeline plus Actos-specific passes
    /// (@mentions and #tags linking).
    /// </summary>
    public static class Actos
    {
        /// <summary>
        /// AST JSON schema version for Actos AST output.
        /// </summary>
        public const int AstSchemaVersion = 1;

        /// <summary>
        /// Returns the markstone engine semver version string.
        /// </summary>
        public static string Version => Markstone.Version;

        /// <summary>
        /// Parses Actos markdown (with @mention and #tag linking) and renders safe HTML.
        /// </summary>
        /// <param name="input">Markdown input text.</param>
        /// <returns>Safe HTML output string with Actos mention and tag links.</returns>
        /// <exception cref="ArgumentNullException">Thrown when <paramref name="input"/> is null.</exception>
        /// <exception cref="InputTooLargeException">Thrown when input exceeds 4 MiB.</exception>
        /// <exception cref="DepthExceededException">Thrown when block nesting depth exceeds 64.</exception>
        /// <exception cref="InvalidUtf8Exception">Thrown when input contains invalid UTF-8 byte sequences.</exception>
        /// <exception cref="MarkstoneException">Thrown on internal error or panic.</exception>
        public static string ToHtml(string input) => ConvertString(NativeOperation.ActosHtml, input);

        /// <summary>
        /// Parses Actos markdown from raw UTF-8 bytes (with @mention and #tag linking) and renders safe HTML.
        /// </summary>
        /// <param name="utf8Input">UTF-8 encoded markdown input bytes.</param>
        /// <returns>Safe HTML output string with Actos mention and tag links.</returns>
        /// <exception cref="InputTooLargeException">Thrown when input exceeds 4 MiB.</exception>
        /// <exception cref="DepthExceededException">Thrown when block nesting depth exceeds 64.</exception>
        /// <exception cref="InvalidUtf8Exception">Thrown when input contains invalid UTF-8 byte sequences.</exception>
        /// <exception cref="MarkstoneException">Thrown on internal error or panic.</exception>
        public static string ToHtml(ReadOnlySpan<byte> utf8Input) => Convert(NativeOperation.ActosHtml, utf8Input);

        /// <summary>
        /// Parses Actos markdown (with @mention and #tag nodes) and produces the AST JSON representation.
        /// </summary>
        /// <param name="input">Markdown input text.</param>
        /// <returns>AST JSON representation with mention and tag nodes.</returns>
        /// <exception cref="ArgumentNullException">Thrown when <paramref name="input"/> is null.</exception>
        /// <exception cref="InputTooLargeException">Thrown when input exceeds 4 MiB.</exception>
        /// <exception cref="DepthExceededException">Thrown when block nesting depth exceeds 64.</exception>
        /// <exception cref="InvalidUtf8Exception">Thrown when input contains invalid UTF-8 byte sequences.</exception>
        /// <exception cref="MarkstoneException">Thrown on internal error or panic.</exception>
        public static string ToAst(string input) => ConvertString(NativeOperation.ActosAst, input);

        /// <summary>
        /// Parses Actos markdown from raw UTF-8 bytes (with @mention and #tag nodes) and produces the AST JSON representation.
        /// </summary>
        /// <param name="utf8Input">UTF-8 encoded markdown input bytes.</param>
        /// <returns>AST JSON representation with mention and tag nodes.</returns>
        /// <exception cref="InputTooLargeException">Thrown when input exceeds 4 MiB.</exception>
        /// <exception cref="DepthExceededException">Thrown when block nesting depth exceeds 64.</exception>
        /// <exception cref="InvalidUtf8Exception">Thrown when input contains invalid UTF-8 byte sequences.</exception>
        /// <exception cref="MarkstoneException">Thrown on internal error or panic.</exception>
        public static string ToAst(ReadOnlySpan<byte> utf8Input) => Convert(NativeOperation.ActosAst, utf8Input);
    }
}
