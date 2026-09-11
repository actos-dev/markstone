using System.Text;
using Markstone;

namespace Markstone.ConformanceRunner;

public static class Program
{
    public static int Main(string[] args)
    {
        string? mode = null;
        string? inputPath = null;

        for (int i = 0; i < args.Length; i++)
        {
            if (args[i] == "--mode" && i + 1 < args.Length)
            {
                mode = args[++i];
            }
            else if (args[i].StartsWith("--mode="))
            {
                mode = args[i]["--mode=".Length..];
            }
            else if (!args[i].StartsWith("--"))
            {
                inputPath = args[i];
            }
        }

        if (string.IsNullOrEmpty(mode))
        {
            Console.Error.WriteLine("Error: --mode argument is required");
            return 1;
        }

        string input;
        if (!string.IsNullOrEmpty(inputPath) && inputPath != "-")
        {
            input = File.ReadAllText(inputPath, Encoding.UTF8);
        }
        else
        {
            using var reader = new StreamReader(Console.OpenStandardInput(), Encoding.UTF8);
            input = reader.ReadToEnd();
        }

        string output = mode switch
        {
            "generic-html" => Markstone.ToHtml(input),
            "generic-ast" => Markstone.ToAst(input),
            "actos-html" => Markstone.Actos.ToHtml(input),
            "actos-ast" => Markstone.Actos.ToAst(input),
            _ => throw new ArgumentException($"Unknown mode: {mode}")
        };

        byte[] outputBytes = Encoding.UTF8.GetBytes(output);
        using var stdout = Console.OpenStandardOutput();
        stdout.Write(outputBytes, 0, outputBytes.Length);
        stdout.Flush();

        return 0;
    }
}
