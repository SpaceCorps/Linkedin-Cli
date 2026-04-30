# Linkedin.Console

CLI for fetching LinkedIn profiles via Apify. YAML-first output optimized for LLM agent consumption.

## Installation

```bash
dotnet tool install -g Linkedin.Console
```

## Prerequisites

Get your API token at [Apify](https://www.apify.com?fpr=ual7wl) and set it:

```bash
export APIFY_TOKEN=your-token-here
```

Or pass it per-command with `--api-key`.

## Usage

```bash
# Fetch a full profile
linkedin profile https://www.linkedin.com/in/username

# Fetch only specific sections
linkedin profile https://www.linkedin.com/in/username --include experiences,skills,educations

# With inline API key
linkedin profile https://www.linkedin.com/in/username --api-key apify_api_xxx
```

### Available sections for `--include`

`experiences`, `skills`, `educations`, `languages`, `licenseAndCertificates`, `honorsAndAwards`, `volunteerAndAwards`, `projects`, `publications`, `patents`, `courses`, `testScores`, `organizations`, `interests`, `recommendations`, `updates`, `profilePicAllDimensions`, `verifications`, `promos`, `highlights`, `volunteerCauses`

## License

MIT
