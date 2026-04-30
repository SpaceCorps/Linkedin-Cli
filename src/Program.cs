using Linkedin.Console.Commands;
using Spectre.Console.Cli;

var app = new CommandApp();

app.Configure(config =>
{
    config.SetApplicationName("linkedin");

    config.AddCommand<ProfileCommand>("profile")
        .WithDescription("Fetch a LinkedIn profile and output as YAML");

    config.AddCommand<PostCommand>("post")
        .WithDescription("Fetch LinkedIn posts and output as YAML");
});

return app.Run(args);
