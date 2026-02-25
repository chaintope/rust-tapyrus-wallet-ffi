using System;
using System.Globalization;
using System.Windows;
using System.Windows.Data;

namespace TapyrusWalletExample
{
    public partial class MainWindow : Window
    {
        private readonly TapyrusWalletManager _walletManager;

        public MainWindow()
        {
            InitializeComponent();

            _walletManager = new TapyrusWalletManager(Dispatcher);
            DataContext = _walletManager;

            Loaded += MainWindow_Loaded;
        }

        private async void MainWindow_Loaded(object sender, RoutedEventArgs e)
        {
            await _walletManager.InitializeAsync();
            await _walletManager.SyncWalletAsync();
        }

        private void GenerateCopyAddress_Click(object sender, RoutedEventArgs e)
        {
            var address = _walletManager.GetNewAddress();
            if (!string.IsNullOrEmpty(address))
            {
                Clipboard.SetText(address);
                MessageBox.Show("The address has been copied to clipboard.",
                    "Address Copied", MessageBoxButton.OK, MessageBoxImage.Information);
            }
        }

        private async void SyncWallet_Click(object sender, RoutedEventArgs e)
        {
            await _walletManager.SyncWalletAsync();
        }

        private void Send_Click(object sender, RoutedEventArgs e)
        {
            if (_walletManager.Balance <= 0)
            {
                MessageBox.Show("No balance available to send.",
                    "Cannot Send", MessageBoxButton.OK, MessageBoxImage.Warning);
                return;
            }

            var transferWindow = new TransferWindow(_walletManager) { Owner = this };
            transferWindow.ShowDialog();
        }

        private async void Settings_Click(object sender, RoutedEventArgs e)
        {
            var settingsWindow = new SettingsWindow() { Owner = this };
            if (settingsWindow.ShowDialog() == true)
            {
                await _walletManager.ReinitializeAsync();
                await _walletManager.SyncWalletAsync();
            }
        }
    }

    public class InverseBoolConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
            => value is bool b ? !b : value;

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
            => value is bool b ? !b : value;
    }

    public class SyncStatusToVisibilityConverter : IValueConverter
    {
        public string TargetStatus { get; set; } = "";

        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is SyncStatus status)
            {
                var target = Enum.TryParse<SyncStatus>(TargetStatus, out var t) ? t : SyncStatus.None;
                return status == target ? Visibility.Visible : Visibility.Collapsed;
            }
            return Visibility.Collapsed;
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
            => throw new NotImplementedException();
    }
}
