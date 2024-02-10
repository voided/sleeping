using Server.Config;
using Server.Identity;
using Server.Services;


var builder = WebApplication.CreateBuilder(args);

// service configuration
builder.Services.AddOptions<ServerOptions>()
    .Bind(builder.Configuration.GetSection(nameof(ServerOptions)));

builder.Services.AddAuthentication(opts =>
{
    opts.AddScheme<SharedSecretAuthenticationHandler>(
        SharedSecretAuthenticationHandler.SchemeName, "Shared Secret"
    );
});

builder.Services.AddControllers();

builder.Services.AddScoped<PingRecorder>();

// --------------

var app = builder.Build();

// app configuration
app.UseAuthentication();
app.UseAuthorization();

if (app.Environment.IsDevelopment())
{
    app.UseDeveloperExceptionPage();
}

app.MapControllers();


app.Run();
