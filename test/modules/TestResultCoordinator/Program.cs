// Copyright (c) Microsoft. All rights reserved.
namespace TestResultCoordinator
{
    using System;
    using System.Threading;
    using System.Threading.Tasks;
    using Microsoft.AspNetCore.Hosting;
    using Microsoft.Azure.Devices.Edge.ModuleUtil;
    using Microsoft.Azure.Devices.Edge.Util;
    using Microsoft.Extensions.Hosting;
    using Microsoft.Extensions.Logging;

    class Program
    {
        static readonly ILogger Logger = ModuleUtil.CreateLogger("TestResultCoordinator");

        static async Task Main(string[] args)
        {
            Logger.LogInformation($"Starting TestResultCoordinator with the following settings:\r\n{Settings.Current}");

            (CancellationTokenSource cts, ManualResetEventSlim completed, Option<object> handler) = ShutdownHandler.Init(TimeSpan.FromSeconds(5), Logger);

            Logger.LogInformation("Creating WebHostBuilder...");
            Task webHost = CreateHostBuilder(args).Build().RunAsync(cts.Token);

            await Task.WhenAny(cts.Token.WhenCanceled(), webHost);

            completed.Set();
            handler.ForEach(h => GC.KeepAlive(h));
            Logger.LogInformation("TestResultCoordinator Main() exited.");
        }

        static IHostBuilder CreateHostBuilder(string[] args) =>
            Host.CreateDefaultBuilder(args)
                .ConfigureWebHostDefaults(webBuilder =>
                {
                    webBuilder
                        .UseUrls($"http://*:{Settings.Current.WebHostPort}")
                        .UseSockets()
                        .UseStartup<Startup>();
                });
    }
}
