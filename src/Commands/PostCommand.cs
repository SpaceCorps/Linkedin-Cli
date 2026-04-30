using System.ComponentModel;
using System.Text.Json;
using Linkedin.Console.Infrastructure;
using Spectre.Console;
using Spectre.Console.Cli;

namespace Linkedin.Console.Commands;

public sealed class PostCommand : AsyncCommand<PostCommand.Settings>
{
    public sealed class Settings : GlobalSettings
    {
        [CommandArgument(0, "<URL>")]
        [Description("LinkedIn URL — post, profile, company, or search URL")]
        public required string Url { get; init; }

        [CommandOption("--limit <N>")]
        [Description("Max posts per source URL (default: unlimited)")]
        public int? Limit { get; init; }

        [CommandOption("--since <DATE>")]
        [Description("Only include posts newer than this date (e.g. 2025-01-01)")]
        public string? Since { get; init; }

        [CommandOption("--no-deep")]
        [Description("Skip additional info (likes, comments, etc.)")]
        public bool NoDeep { get; init; }

        [CommandOption("--raw")]
        [Description("Return raw unprocessed data from the scraper")]
        public bool Raw { get; init; }

        [CommandOption("--include <SECTIONS>")]
        [Description("Comma-separated fields to include in output")]
        public string? Include { get; init; }
    }

    protected override async Task<int> ExecuteAsync(CommandContext context, Settings settings, CancellationToken cancellation)
    {
        var url = LinkedInUrl.NormalizePost(settings.Url);

        AnsiConsole.MarkupLine($"[grey]Fetching posts (this may take 30–60s)...[/]");

        using var client = settings.CreateClient();
        var doc = await client.FetchPostsAsync(url, settings.Limit, settings.Since, !settings.NoDeep, settings.Raw);

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
