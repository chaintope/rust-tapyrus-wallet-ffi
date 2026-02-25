using System;
using System.ComponentModel;
using System.IO;
using System.Runtime.CompilerServices;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Threading;
using com.chaintope.tapyrus.wallet;

namespace TapyrusWalletExample
{
    public enum SyncStatus
    {
        None,
        Success,
        Failure
    }

    public class TapyrusWalletManager : INotifyPropertyChanged
    {
        public event PropertyChangedEventHandler? PropertyChanged;

        private string _currentAddress = "";
        public string CurrentAddress
        {
            get => _currentAddress;
            private set { _currentAddress = value; OnPropertyChanged(); }
        }

        private double _balance;
        public double Balance
        {
            get => _balance;
            private set { _balance = value; OnPropertyChanged(); }
        }

        private bool _isSyncing;
        public bool IsSyncing
        {
            get => _isSyncing;
            private set { _isSyncing = value; OnPropertyChanged(); }
        }

        private string _connectionInfo = "";
        public string ConnectionInfo
        {
            get => _connectionInfo;
            private set { _connectionInfo = value; OnPropertyChanged(); }
        }

        private SyncStatus _syncStatus = SyncStatus.None;
        public SyncStatus SyncStatus
        {
            get => _syncStatus;
            private set { _syncStatus = value; OnPropertyChanged(); }
        }

        private string? _syncErrorMessage;
        public string? SyncErrorMessage
        {
            get => _syncErrorMessage;
            private set { _syncErrorMessage = value; OnPropertyChanged(); }
        }

        private string? _errorMessage;
        public string? ErrorMessage
        {
            get => _errorMessage;
            private set { _errorMessage = value; OnPropertyChanged(); }
        }

        private HdWallet? _wallet;
        private readonly Network _networkMode = Network.Prod;
        private readonly Dispatcher _dispatcher;

        private static readonly string AppDir = AppDomain.CurrentDomain.BaseDirectory;
        private static readonly string MasterKeyPath = Path.Combine(AppDir, "master_key");
        private static readonly string DbFilePath = Path.Combine(AppDir, "tapyrus_wallet.db");
        private static readonly string SettingsPath = Path.Combine(AppDir, "settings.json");

        public TapyrusWalletManager(Dispatcher dispatcher)
        {
            _dispatcher = dispatcher;
        }

        public async Task InitializeAsync()
        {
            await Task.Run(() => SetupWallet());
        }

        private void SetupWallet()
        {
            try
            {
                // Load or generate master key
                string masterKey;
                if (File.Exists(MasterKeyPath))
                {
                    masterKey = File.ReadAllText(MasterKeyPath).Trim();
                }
                else
                {
                    masterKey = WalletMethods.GenerateMasterKey(_networkMode);
                    File.WriteAllText(MasterKeyPath, masterKey);
                }

                // Load connection settings
                var settings = AppSettings.Load(SettingsPath);

                Config config;
                if (settings.ConnectionType == ConnectionType.Electrum)
                {
                    var host = settings.ElectrumHost;
                    var port = ushort.TryParse(settings.ElectrumPort, out var p) ? p : (ushort)50001;

                    config = new Config(
                        networkMode: _networkMode,
                        networkId: 1939510133,
                        genesisHash: "038b114875c2f78f5a2fd7d8549a905f38ea5faee6e29a3d79e547151d6bdd8a",
                        electrumDomain: host,
                        electrumPort: port,
                        masterKey: masterKey,
                        dbFilePath: DbFilePath);

                    _dispatcher.Invoke(() => ConnectionInfo = $"Electrum: {host}:{port}");
                }
                else
                {
                    var host = settings.EsploraHost;
                    var portStr = settings.EsploraPort;
                    var esploraUrl = $"http://{host}:{portStr}";

                    config = new Config(
                        networkMode: _networkMode,
                        networkId: 1939510133,
                        genesisHash: "038b114875c2f78f5a2fd7d8549a905f38ea5faee6e29a3d79e547151d6bdd8a",
                        esploraUrl: esploraUrl,
                        masterKey: masterKey,
                        dbFilePath: DbFilePath);

                    _dispatcher.Invoke(() => ConnectionInfo = $"Esplora: {esploraUrl}");
                }

                _wallet = new HdWallet(config: config);
            }
            catch (Exception ex)
            {
                _dispatcher.Invoke(() => ErrorMessage = $"Failed to initialize wallet: {ex.Message}");
            }
        }

        public async Task SyncWalletAsync()
        {
            if (_wallet == null || IsSyncing) return;

            _dispatcher.Invoke(() =>
            {
                IsSyncing = true;
                SyncStatus = SyncStatus.None;
                SyncErrorMessage = null;
            });

            await Task.Run(() =>
            {
                try
                {
                    _wallet.FullSync();
                    UpdateBalance();

                    _dispatcher.Invoke(() =>
                    {
                        IsSyncing = false;
                        SyncStatus = SyncStatus.Success;
                    });
                }
                catch (Exception ex)
                {
                    _dispatcher.Invoke(() =>
                    {
                        IsSyncing = false;
                        SyncStatus = SyncStatus.Failure;
                        SyncErrorMessage = ex.Message;
                    });
                }
            });
        }

        public string GetNewAddress()
        {
            if (_wallet == null) return "";

            try
            {
                var result = _wallet.GetNewAddress(null);
                var address = result.address;
                _dispatcher.Invoke(() => CurrentAddress = address);
                return address;
            }
            catch (Exception ex)
            {
                _dispatcher.Invoke(() => ErrorMessage = $"Failed to generate address: {ex.Message}");
                return "";
            }
        }

        public async Task<string> TransferAsync(string toAddress, double amount)
        {
            if (_wallet == null)
                throw new InvalidOperationException("Wallet is not initialized");

            return await Task.Run(() =>
            {
                var amountInSatoshis = (ulong)(amount * 100_000_000.0);
                var transferParams = new TransferParams(amount: amountInSatoshis, toAddress: toAddress);
                var txid = _wallet.Transfer([transferParams], []);

                UpdateBalance();
                return txid;
            });
        }

        public async Task ReinitializeAsync()
        {
            _wallet = null;

            _dispatcher.Invoke(() =>
            {
                CurrentAddress = "";
                Balance = 0.0;
                SyncStatus = SyncStatus.None;
                SyncErrorMessage = null;
            });

            // Delete old DB to start fresh with new connection
            if (File.Exists(DbFilePath))
            {
                try { File.Delete(DbFilePath); } catch { }
            }

            await Task.Run(() => SetupWallet());
        }

        private void UpdateBalance()
        {
            if (_wallet == null) return;

            try
            {
                var balanceValue = _wallet.Balance(null);
                var tpcBalance = (double)balanceValue / 100_000_000.0;
                _dispatcher.Invoke(() => Balance = tpcBalance);
            }
            catch (Exception ex)
            {
                _dispatcher.Invoke(() => ErrorMessage = $"Failed to get balance: {ex.Message}");
            }
        }

        private void OnPropertyChanged([CallerMemberName] string? propertyName = null)
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }
    }
}
