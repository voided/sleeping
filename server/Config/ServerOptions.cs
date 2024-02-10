namespace Server.Config
{
    public class ServerOptions
    {
        /// <summary>
        /// Where to store the CSV files containing ping timestamps.
        /// </summary>
        public string? DataDirectory { get; set; }

        /// <summary>
        /// The shared secret to use for authenticating requests.
        /// </summary>
        public string? SharedSecret { get; set; }
    }
}
