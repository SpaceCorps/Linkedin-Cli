using System.Text.RegularExpressions;

namespace Linkedin.Console.Infrastructure;

public static partial class LinkedInUrl
{
    public static string NormalizeProfile(string url)
    {
        var match = ProfileRegex().Match(url.Trim());

        if (!match.Success)
            throw new ArgumentException($"Invalid LinkedIn profile URL: {url}");

        return $"https://www.linkedin.com/in/{match.Groups[1].Value}";
    }

    public static string NormalizePost(string url)
    {
        var trimmed = url.Trim();

        if (PostUrlRegex().IsMatch(trimmed) || ProfileRegex().IsMatch(trimmed) || CompanyRegex().IsMatch(trimmed) || SearchRegex().IsMatch(trimmed))
            return trimmed;

        throw new ArgumentException($"Invalid LinkedIn URL: {url}. Expected a post, profile, company, or search URL.");
    }

    [GeneratedRegex(@"(?:https?://)?(?:www\.)?linkedin\.com/in/([^/?\s]+)", RegexOptions.IgnoreCase)]
    private static partial Regex ProfileRegex();

    [GeneratedRegex(@"(?:https?://)?(?:www\.)?linkedin\.com/(feed/update|posts/)", RegexOptions.IgnoreCase)]
    private static partial Regex PostUrlRegex();

    [GeneratedRegex(@"(?:https?://)?(?:www\.)?linkedin\.com/company/", RegexOptions.IgnoreCase)]
    private static partial Regex CompanyRegex();

    [GeneratedRegex(@"(?:https?://)?(?:www\.)?linkedin\.com/search/", RegexOptions.IgnoreCase)]
    private static partial Regex SearchRegex();
}
