using System.Text.RegularExpressions;

namespace Linkedin.Console.Infrastructure;

public static partial class LinkedInUrl
{
    public static string Normalize(string url)
    {
        var match = ProfileRegex().Match(url.Trim());

        if (!match.Success)
            throw new ArgumentException($"Invalid LinkedIn profile URL: {url}");

        return $"https://www.linkedin.com/in/{match.Groups[1].Value}";
    }

    [GeneratedRegex(@"(?:https?://)?(?:www\.)?linkedin\.com/in/([^/?\s]+)", RegexOptions.IgnoreCase)]
    private static partial Regex ProfileRegex();
}
