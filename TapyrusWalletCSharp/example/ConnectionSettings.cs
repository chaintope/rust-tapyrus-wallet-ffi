using System;
using System.IO;
using System.Text.Json;

namespace TapyrusWalletExample
{
    public enum ConnectionType
    {
        Esplora,
        Electrum
    }

    public static class DefaultConnection
    {
        public const string EsploraHost = "localhost";
        public const string EsploraPort = "3001";
        public const string ElectrumHost = "localhost";
        public const string ElectrumPort = "50001";
    }

    public class AppSettings
    {
        public ConnectionType ConnectionType { get; set; } = ConnectionType.Esplora;
        public string EsploraHost { get; set; } = DefaultConnection.EsploraHost;
        public string EsploraPort { get; set; } = DefaultConnection.EsploraPort;
        public string ElectrumHost { get; set; } = DefaultConnection.ElectrumHost;
        public string ElectrumPort { get; set; } = DefaultConnection.ElectrumPort;

        private static readonly JsonSerializerOptions JsonOptions = new()
        {
            WriteIndented = true
        };

        public static AppSettings Load(string path)
        {
            if (!File.Exists(path))
                return new AppSettings();

            try
            {
                var json = File.ReadAllText(path);
                return JsonSerializer.Deserialize<AppSettings>(json, JsonOptions) ?? new AppSettings();
            }
            catch
            {
                return new AppSettings();
            }
        }

        public void Save(string path)
        {
            var json = JsonSerializer.Serialize(this, JsonOptions);
            File.WriteAllText(path, json);
        }
    }
}
