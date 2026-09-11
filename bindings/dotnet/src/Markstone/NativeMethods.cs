using System.Runtime.InteropServices;

namespace Markstone;

/// <summary>
/// P/Invoke declarations for markstone C ABI via [LibraryImport].
/// </summary>
internal static partial class NativeMethods
{
    internal const string LibraryName = "markstone_abi";

    static NativeMethods()
    {
        NativeLibraryResolver.EnsureInstalled();
    }

    [LibraryImport(LibraryName, EntryPoint = "markstone_to_html")]
    internal static unsafe partial int markstone_to_html(
        byte* input,
        nuint input_len,
        out IntPtr @out,
        out nuint out_len);

    [LibraryImport(LibraryName, EntryPoint = "markstone_to_ast")]
    internal static unsafe partial int markstone_to_ast(
        byte* input,
        nuint input_len,
        out IntPtr @out,
        out nuint out_len);

    [LibraryImport(LibraryName, EntryPoint = "markstone_actos_to_html")]
    internal static unsafe partial int markstone_actos_to_html(
        byte* input,
        nuint input_len,
        out IntPtr @out,
        out nuint out_len);

    [LibraryImport(LibraryName, EntryPoint = "markstone_actos_to_ast")]
    internal static unsafe partial int markstone_actos_to_ast(
        byte* input,
        nuint input_len,
        out IntPtr @out,
        out nuint out_len);

    [LibraryImport(LibraryName, EntryPoint = "markstone_free")]
    internal static partial void markstone_free(IntPtr ptr, nuint len);

    [LibraryImport(LibraryName, EntryPoint = "markstone_version")]
    internal static partial IntPtr markstone_version();

    [LibraryImport(LibraryName, EntryPoint = "markstone_ast_schema_version")]
    internal static partial uint markstone_ast_schema_version();
}
