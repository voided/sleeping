using Microsoft.AspNetCore.Authentication;
using Microsoft.Extensions.Options;
using Microsoft.Net.Http.Headers;
using Server.Config;
using System.Diagnostics;
using System.Net.Http.Headers;
using System.Security.Principal;
using System.Text.Encodings.Web;

namespace Server.Identity
{
    public class SharedSecretAuthenticationHandler : AuthenticationHandler<AuthenticationSchemeOptions>
    {
        public const string SchemeName = "SharedSecret";


        private readonly ServerOptions _serverOptions;


        public SharedSecretAuthenticationHandler(
            IOptionsMonitor<AuthenticationSchemeOptions> options, ILoggerFactory logger, UrlEncoder encoder,
            IOptions<ServerOptions> serverOptions
        )
            : base(options, logger, encoder)
        {
            Debug.Assert(serverOptions != null);

            _serverOptions = serverOptions.Value;
        }


        protected override Task<AuthenticateResult> HandleAuthenticateAsync()
        {
            if (!Request.Headers.ContainsKey(HeaderNames.Authorization))
            {
                return Task.FromResult(AuthenticateResult.NoResult());
            }

            if (!AuthenticationHeaderValue.TryParse(Request.Headers.Authorization, out AuthenticationHeaderValue? authorizationHeaderValues))
            {
                return Task.FromResult(AuthenticateResult.Fail("Invalid Authorization header"));
            }

            if (authorizationHeaderValues.Scheme != SchemeName)
            {
                // auth header specified isn't for this auth handler
                return Task.FromResult(AuthenticateResult.NoResult());
            }

            if (_serverOptions.SharedSecret != authorizationHeaderValues.Parameter)
            {
                return Task.FromResult(AuthenticateResult.Fail("Invalid shared secret"));
            }

            var identity = new GenericIdentity("user");
            var princiapl = new GenericPrincipal(identity, null);
            var ticket = new AuthenticationTicket(princiapl, Scheme.Name);

            return Task.FromResult(AuthenticateResult.Success(ticket));
        }
    }
}
