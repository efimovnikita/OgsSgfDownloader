using System;
using System.Collections.Generic;
using System.CommandLine;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;
using Newtonsoft.Json;
using OgsSgfDownloader.Models;
using Serilog;

namespace OgsSgfDownloader
{
    internal static class Program
    {
        private const string Url = "https://online-go.com";
        private static readonly ILogger Logger = Log.Logger = new LoggerConfiguration()
            .WriteTo.Console()
            .CreateLogger();
        private static async Task<int> Main(string[] args)
        {
            (Option<List<string>> playersOption, Option<List<int>> rangeOption, Option<string> pathOption) = MakeOptions();

            RootCommand rootCommand = new("Uploader of sgf files from the OGS server");
            rootCommand.AddOption(playersOption);
            rootCommand.AddOption(rangeOption);
            rootCommand.AddOption(pathOption);

            rootCommand.SetHandler(async (names, range, path) =>
            {
                foreach (string name in names)
                {
                    await DownloadGamesForPlayer(name, range, path);
                }
            }, playersOption, rangeOption, pathOption);

            return await rootCommand.InvokeAsync(args);
        }

        private static async Task DownloadGamesForPlayer(string name, List<int> range, string path)
        {
            HttpClientHandler clientHandler = new()
            {
                UseCookies = false
            };
            HttpClient client = new(clientHandler);
            List<string> gamesDetails = await Get9X9Games(client, name, range[0], range[1]);
            
            path = Path.Combine(path, await GetPlayerName(client, name)); // add name to path

            await DownloadSgf(client, gamesDetails, path);
        }

        private static async Task<string> GetPlayerName(HttpClient client, string name)
        {
            HttpRequestMessage request = new()
            {
                Method = HttpMethod.Get,
                RequestUri = new Uri($"{Url}/api/v1/players{name}")
            };
            using HttpResponseMessage response = await client.SendAsync(request);

            if (response.IsSuccessStatusCode == false)
            {
                Logger.Warning("Getting player details unsuccessful");
                return String.Empty;
            }

            string body = await response.Content.ReadAsStringAsync();
            Player player;
            try
            {
                player = JsonConvert.DeserializeObject<Player>(body);
            }
            catch (Exception exception)
            {
                Logger.Error(exception,"Error while deserialization player details");
                return String.Empty;
            }
            
            return player?.Username ?? String.Empty;
        }

        private static (Option<List<string>> playersOption, Option<List<int>> rangeOption, Option<string> pathOption) MakeOptions()
        {
            Option<List<string>> playersOption = new("--players", "Player id from OGS server. Example: -p 64817")
            {
                IsRequired = true,
                AllowMultipleArgumentsPerToken = true
            };
            playersOption.AddAlias("-p");

            Option<List<int>> rangeOption = new("--range", @"Range of downloadable games. Example: -r 1 100")
            {
                IsRequired = true,
                AllowMultipleArgumentsPerToken = true
            };
            rangeOption.AddAlias("-r");

            Option<string> pathOption =
                new("--path", @"Folder for saving sgf files. Example: --path /home/maskedball/Downloads/9x9SGF")
                {
                    IsRequired = true
                };
            return (playersOption, rangeOption, pathOption);
        }

        private static async Task DownloadSgf(HttpClient client, List<string> details, string path)
        {
            if (Directory.Exists(path) == false)
            {
                Logger.Information("Directory doesn't exist. Create directory");
                Directory.CreateDirectory(path!);
            }
            
            foreach ((string Detail, int Index) detailWithIndex in details.Select((s, i) => (s, i)))
            {
                string fullPath = $"{path}/{detailWithIndex.Detail.Split('/').Last()}.sgf";
                if (File.Exists(fullPath))
                {
                    Logger.Information("File \"{Filename}\" already exists. Skip", Path.GetFileName(fullPath));
                    continue;
                }

                HttpRequestMessage request = new()
                {
                    Method = HttpMethod.Get,
                    RequestUri = new Uri($"{Url}/{detailWithIndex.Detail}/sgf")
                };
                using HttpResponseMessage response = await client.SendAsync(request);

                if (response.IsSuccessStatusCode == false)
                {
                    Logger.Warning("SGF download at address {Address} is unsuccessful (code - {Code})",
                        request.RequestUri, response.StatusCode);
                    continue;
                }

                string body = await response.Content.ReadAsStringAsync();

                await File.WriteAllTextAsync(fullPath, body);
                Logger.Information("File {Index}/{Count} saved", detailWithIndex.Index + 1, 
                    details.Count);

                Thread.Sleep(TimeSpan.FromMilliseconds(new Random().Next(500, 1000)));
            }
        }

        private static async Task<List<string>> Get9X9Games(HttpClient client, string name, int from, int to)
        {
            List<string> result = new();
            for (int i = from; i < to; i++)
            {
                HttpRequestMessage request = new()
                {
                    Method = HttpMethod.Get,
                    RequestUri = new Uri($"{Url}/api/v1/players{name}/games?page={i}")
                };

                using HttpResponseMessage response = await client.SendAsync(request);
                if (response.IsSuccessStatusCode == false)
                {
                    Logger.Warning("Games download at page {Address} is unsuccessful (code - {Code})",
                        request.RequestUri, response.StatusCode);
                    continue;
                }

                Logger.Information("Download games from page {Number}. Url: {Url}", i, request.RequestUri);

                string body = await response.Content.ReadAsStringAsync();
                Root page;
                try
                {
                    page = JsonConvert.DeserializeObject<Root>(body);
                }
                catch (Exception exception)
                {
                    Logger.Error(exception, "Error occured during page deserialization");
                    continue;
                }

                result
                    .AddRange(page!.Results
                        .Where(pageResult => pageResult.Width == 9)
                        .Select(pageResult => pageResult.Related.Detail));
                
                Thread.Sleep(TimeSpan.FromMilliseconds(new Random().Next(500, 1000)));
            }

            return result;
        }
    }
}