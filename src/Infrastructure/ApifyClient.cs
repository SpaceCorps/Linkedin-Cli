using System.Net.Http.Json;
using System.Text.Json;

namespace Linkedin.Console.Infrastructure;

public sealed class ApifyClient : IDisposable
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        PropertyNameCaseInsensitive = true,
        DefaultIgnoreCondition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull
    };

    private readonly HttpClient _http;
    private readonly string _token;

    public ApifyClient(string token)
    {
        _token = token;
        _http = new HttpClient
        {
            BaseAddress = new Uri("https://api.apify.com/v2/"),
            Timeout = TimeSpan.FromMinutes(5)
        };
    }

    public async Task<JsonDocument> FetchProfileAsync(string profileUrl)
    {
        var endpoint = $"acts/dev_fusion~Linkedin-Profile-Scraper/run-sync-get-dataset-items?token={_token}";
        var body = new { profileUrls = new[] { profileUrl } };

        return await PostAndReadAsync(endpoint, body);
    }

    public async Task<JsonDocument> FetchPostsAsync(
        string url, int? limit = null, string? since = null, bool deepScrape = true, bool raw = false)
    {
        var endpoint = $"acts/supreme_coder~linkedin-post/run-sync-get-dataset-items?token={_token}";
        var body = new
        {
            urls = new[] { url },
            limitPerSource = limit,
            scrapeUntil = since,
            deepScrape,
            rawData = raw
        };

        return await PostAndReadAsync(endpoint, body);
    }

    private async Task<JsonDocument> PostAndReadAsync(string endpoint, object body)
    {
        var response = await _http.PostAsJsonAsync(endpoint, body, JsonOptions);

        if (!response.IsSuccessStatusCode)
        {
            var error = await response.Content.ReadAsStringAsync();
            throw new HttpRequestException($"Apify API error: {response.StatusCode} — {error}");
        }

        var stream = await response.Content.ReadAsStreamAsync();
        return await JsonDocument.ParseAsync(stream);
    }

    public void Dispose() => _http.Dispose();
}
