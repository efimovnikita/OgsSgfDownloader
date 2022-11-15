using System.CommandLine;
using System.Diagnostics;
using Telegram.Bot;
using Telegram.Bot.Types;
using Telegram.Bot.Types.InputFiles;
using File = System.IO.File;

namespace OgsSgfDownloaderTelegramBot
{
    internal static class Program
    {
        private static string? _downloadPath;

        private static async Task<int> Main(string[] args)
        {
            Option<string> apiOption = new("--api", "API key for telegram bot")
            {
                IsRequired = true,
            };
            
            Option<string> pathOption =
                new("--path", @"Folder for saving sgf files. Example: --path /home/maskedball/Downloads/9x9SGF")
                {
                    IsRequired = true
                };
            
            RootCommand rootCommand = new("Telegram bot for SGF downloader");
            rootCommand.AddOption(apiOption);
            rootCommand.AddOption(pathOption);
            
            rootCommand.SetHandler((api, path) =>
            {
                _downloadPath = path;
                Run(api);
            }, apiOption, pathOption);

            return await rootCommand.InvokeAsync(args);
        }

        private static void Run(string api)
        {
            TelegramBotClient client = new(api);
            client.StartReceiving(Update, Error);

            Console.ReadLine();
        }

        private static Task Error(ITelegramBotClient arg1, Exception arg2, CancellationToken arg3)
        {
            Console.WriteLine(arg2);
            return Task.CompletedTask;
        }

        private static async Task Update(ITelegramBotClient client, Update update, CancellationToken token)
        {
            Message? message = update.Message;

            if (message is null)
            {
                return;
            }

            string? text = message.Text;
            
            if (new [] { _downloadPath, text }.Any(String.IsNullOrEmpty))
            {
                return;
            }
            
            long chatId = message.Chat.Id;
            if (text!.Equals("/start"))
            {
                await client.SendTextMessageAsync(chatId, 
                    "Please enter OGS-server player id (Example: /player 64817)",
                    cancellationToken: token);
                return;
            }
            
            if (text.Equals("/player"))
            {
                await client.SendTextMessageAsync(chatId, 
                    "Please provide player id (Example: /player 64817)",
                    cancellationToken: token);
                return;
            }
            
            if (text.StartsWith("/player"))
            {
                Task.Factory.StartNew(() =>
                {
                    ProcessPlayer(client, token, text, chatId);
                });
            }
        }

        private static async Task ProcessPlayer(ITelegramBotClient client, CancellationToken token, string text, long chatId)
        {
            string[] split = text.Split(' ');
            if (split.Length < 2)
            {
                return;
            }

            bool parseResult = Int32.TryParse(split[1], out int id);
            if (parseResult == false)
            {
                return;
            }

            await client.SendTextMessageAsync(chatId,
                $"Downloading games for player with id \'{id}\'. After downloading I send you sgf files.",
                cancellationToken: token);

            string guid = Guid.NewGuid().ToString();
            string fullPath = Path.Combine(_downloadPath!, guid);

            Process process = new();
            process.StartInfo.Arguments = $"-p {id} -r 1 4 --path {fullPath}";
            process.StartInfo.FileName = "SGFdownloader";
            process.Start();
            await process.WaitForExitAsync(token);

            string dirWithPlayerGames = Directory.GetDirectories(fullPath).First();

            string[] sgfFiles = Directory.GetFiles(dirWithPlayerGames);
            foreach (string fileName in sgfFiles)
            {
                await using FileStream stream = File.OpenRead(fileName);
                await client.SendDocumentAsync(chatId, new InputOnlineFile(stream, fileName), cancellationToken: token);
            }

            await client.SendTextMessageAsync(chatId, $"Done. Games for player '{new DirectoryInfo(dirWithPlayerGames).Name}' send to you.", cancellationToken: token);
        }
    }
}