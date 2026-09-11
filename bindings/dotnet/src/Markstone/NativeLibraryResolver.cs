using System.Reflection;
using System.Runtime.InteropServices;

namespace Markstone;

/// <summary>
/// Dynamic resolver for the markstone native library.
/// </summary>
internal static class NativeLibraryResolver
{
    private static int _installed;

    internal static void EnsureInstalled()
    {
        if (Interlocked.Exchange(ref _installed, 1) == 0)
        {
            NativeLibrary.SetDllImportResolver(typeof(NativeMethods).Assembly, Resolve);
        }
    }

    private static IntPtr Resolve(string libraryName, Assembly assembly, DllImportSearchPath? searchPath)
    {
        if (libraryName != NativeMethods.LibraryName && libraryName != "markstone")
        {
            return IntPtr.Zero;
        }

        // 1. Runtime / NuGet runtimes/{rid}/native/
        if (NativeLibrary.TryLoad(libraryName, assembly, searchPath, out var handle))
        {
            return handle;
        }

        var (primaryName, fallbackName) = GetLibraryNames();
        var rid = GetRuntimeIdentifier();

        // Check runtimes/{rid}/native/ adjacent to the loaded assembly
        var baseDir = AppContext.BaseDirectory;
        if (!string.IsNullOrEmpty(baseDir))
        {
            foreach (var name in new[] { primaryName, fallbackName })
            {
                var ridPath = Path.Combine(baseDir, "runtimes", rid, "native", name);
                if (File.Exists(ridPath) && NativeLibrary.TryLoad(ridPath, out handle))
                {
                    return handle;
                }
            }
        }

        // 2. MARKSTONE_LIBRARY_PATH or MARKSTONE_NATIVE_DIR environment variables
        var envLibPath = Environment.GetEnvironmentVariable("MARKSTONE_LIBRARY_PATH");
        if (!string.IsNullOrWhiteSpace(envLibPath))
        {
            if (File.Exists(envLibPath) && NativeLibrary.TryLoad(envLibPath, out handle))
            {
                return handle;
            }

            if (Directory.Exists(envLibPath))
            {
                foreach (var name in new[] { primaryName, fallbackName })
                {
                    var p = Path.Combine(envLibPath, name);
                    if (File.Exists(p) && NativeLibrary.TryLoad(p, out handle))
                    {
                        return handle;
                    }
                }
            }
        }

        var envNativeDir = Environment.GetEnvironmentVariable("MARKSTONE_NATIVE_DIR");
        if (!string.IsNullOrWhiteSpace(envNativeDir))
        {
            if (File.Exists(envNativeDir) && NativeLibrary.TryLoad(envNativeDir, out handle))
            {
                return handle;
            }

            if (Directory.Exists(envNativeDir))
            {
                foreach (var name in new[] { primaryName, fallbackName })
                {
                    var p = Path.Combine(envNativeDir, name);
                    if (File.Exists(p) && NativeLibrary.TryLoad(p, out handle))
                    {
                        return handle;
                    }
                }
            }
        }

        // 3. Walk up from assembly base directory to probe repository target directories
        var probeDir = Path.GetDirectoryName(AppContext.BaseDirectory);
        for (var i = 0; i < 10 && probeDir is not null; i++)
        {
            foreach (var sub in new[] { "target/release", "target/debug", "target" })
            {
                foreach (var name in new[] { primaryName, fallbackName })
                {
                    var candidate = Path.Combine(probeDir, sub, name);
                    if (File.Exists(candidate) && NativeLibrary.TryLoad(candidate, out handle))
                    {
                        return handle;
                    }
                }
            }
            probeDir = Path.GetDirectoryName(probeDir.TrimEnd(Path.DirectorySeparatorChar));
        }

        // Probe current working directory hierarchy as a fallback
        var cwd = Directory.GetCurrentDirectory();
        probeDir = cwd;
        for (var i = 0; i < 10 && probeDir is not null; i++)
        {
            foreach (var sub in new[] { "target/release", "target/debug", "target" })
            {
                foreach (var name in new[] { primaryName, fallbackName })
                {
                    var candidate = Path.Combine(probeDir, sub, name);
                    if (File.Exists(candidate) && NativeLibrary.TryLoad(candidate, out handle))
                    {
                        return handle;
                    }
                }
            }
            probeDir = Path.GetDirectoryName(probeDir.TrimEnd(Path.DirectorySeparatorChar));
        }

        return IntPtr.Zero;
    }

    private static (string Primary, string Fallback) GetLibraryNames()
    {
        if (OperatingSystem.IsWindows())
        {
            return ("markstone_abi.dll", "markstone.dll");
        }
        if (OperatingSystem.IsMacOS())
        {
            return ("libmarkstone_abi.dylib", "libmarkstone.dylib");
        }
        return ("libmarkstone_abi.so", "libmarkstone.so");
    }

    private static string GetRuntimeIdentifier()
    {
        var arch = RuntimeInformation.ProcessArchitecture switch
        {
            Architecture.X64 => "x64",
            Architecture.Arm64 => "arm64",
            Architecture.X86 => "x86",
            Architecture.Arm => "arm",
            _ => "x64"
        };

        if (OperatingSystem.IsWindows()) return $"win-{arch}";
        if (OperatingSystem.IsMacOS()) return $"osx-{arch}";
        return $"linux-{arch}";
    }
}
