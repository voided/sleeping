using CsvHelper;
using CsvHelper.Configuration;
using Microsoft.Extensions.Options;
using Server.Config;
using System.Diagnostics;
using System.Globalization;

namespace Server.Services
{
    public class PingRecorder
    {
        private readonly ServerOptions _serverOptions;


        public PingRecorder(IOptions<ServerOptions> serverOptions)
        {
            Debug.Assert(serverOptions != null);

            _serverOptions = serverOptions.Value;

            if (!Directory.Exists(_serverOptions.DataDirectory))
            {
                throw new DirectoryNotFoundException($"Data directory {_serverOptions.DataDirectory} does not exist");
            }
        }


        public async Task<DateTime> Record()
        {
            Debug.Assert(_serverOptions.DataDirectory != null);

            DateTime timeRecorded = DateTime.UtcNow;

            string fileName = $"{timeRecorded:yyyy-MM-dd}.csv";
            string fullFilePath = Path.Combine(_serverOptions.DataDirectory, fileName);

            var csvConfig = new CsvConfiguration(CultureInfo.InvariantCulture)
            {
                // don't write the header
                HasHeaderRecord = false,
            };

            // append to the csv file in the fullFilePath
            await using var streamWriter = new StreamWriter(fullFilePath, true);
            await using var csvWriter = new CsvWriter(streamWriter, csvConfig);

            await csvWriter.WriteRecordsAsync(new[] { new { Time = timeRecorded } });

            return timeRecorded;
        }
    }
}
