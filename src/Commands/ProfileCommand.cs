using System.ComponentModel;
using System.Text.Json;
using Linkedin.Console.Infrastructure;
using Spectre.Console;
using Spectre.Console.Cli;

namespace Linkedin.Console.Commands;

public sealed class ProfileCommand : AsyncCommand<ProfileCommand.Settings>
{
    public sealed class Settings : GlobalSettings
    {
        [CommandArgument(0, "<URL>")]
        [Description("LinkedIn profile URL (e.g. https://www.linkedin.com/in/username)")]
        public required string Url { get; init; }

        [CommandOption("--include <SECTIONS>")]
        [Description("Comma-separated sections to include (e.g. experiences,skills,educations)")]
        public string? Include { get; init; }
    }

    public override async Task<int> ExecuteAsync(CommandContext context, Settings settings)
    {
        var normalizedUrl = LinkedInUrl.Normalize(settings.Url);

        AnsiConsole.MarkupLine($"[grey]Fetching profile (this may take 30–60s)...[/]");

        using var client = settings.CreateClient();
        var doc = await client.FetchProfileAsync(normalizedUrl);

        HashSet<string>? includeFields = null;
        if (!string.IsNullOrWhiteSpace(settings.Include))
        {
            includeFields = new HashSet<string>(
                settings.Include.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries),
                StringComparer.OrdinalIgnoreCase);
        }

        var obj = ConvertJsonElement(doc.RootElement);
        YamlOutput.Write(obj, includeFields);

        return 0;
    }

    private static object? ConvertJsonElement(JsonElement element) => element.ValueKind switch
    {
        JsonValueKind.Object => element.EnumerateObject()
            .ToDictionary(p => p.Name, p => ConvertJsonElement(p.Value)),
        JsonValueKind.Array => element.EnumerateArray().Select(ConvertJsonElement).ToList(),
        JsonValueKind.String => element.GetString(),
        JsonValueKind.Number => element.TryGetInt64(out var l) ? l : element.GetDouble(),
        JsonValueKind.True => true,
        JsonValueKind.False => false,
        _ => null
    };
}
