using System;
using System.Windows;

namespace TapyrusWalletExample
{
    public partial class TransferWindow : Window
    {
        private readonly TapyrusWalletManager _walletManager;
        private bool _isProcessing;

        public TransferWindow(TapyrusWalletManager walletManager)
        {
            InitializeComponent();
            _walletManager = walletManager;

            BalanceText.Text = $"Available Balance: {_walletManager.Balance:F8} TPC";
        }

        private void AmountText_TextChanged(object sender, System.Windows.Controls.TextChangedEventArgs e)
        {
            if (double.TryParse(AmountText.Text, out var amount) && amount > _walletManager.Balance)
            {
                InsufficientText.Visibility = Visibility.Visible;
            }
            else
            {
                InsufficientText.Visibility = Visibility.Collapsed;
            }
        }

        private async void Send_Click(object sender, RoutedEventArgs e)
        {
            var address = AddressText.Text.Trim();
            var amountStr = AmountText.Text.Trim();

            if (string.IsNullOrEmpty(address))
            {
                MessageBox.Show("Please enter a recipient address.",
                    "Validation Error", MessageBoxButton.OK, MessageBoxImage.Warning);
                return;
            }

            if (!double.TryParse(amountStr, out var amount) || amount <= 0)
            {
                MessageBox.Show("Please enter a valid positive amount.",
                    "Validation Error", MessageBoxButton.OK, MessageBoxImage.Warning);
                return;
            }

            if (amount > _walletManager.Balance)
            {
                MessageBox.Show("Insufficient balance.",
                    "Validation Error", MessageBoxButton.OK, MessageBoxImage.Warning);
                return;
            }

            SetProcessing(true);

            try
            {
                var txid = await _walletManager.TransferAsync(address, amount);

                MessageBox.Show($"Transaction ID:\n{txid}",
                    "Transfer Successful", MessageBoxButton.OK, MessageBoxImage.Information);

                DialogResult = true;
                Close();
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Transfer failed:\n{ex.Message}",
                    "Transfer Error", MessageBoxButton.OK, MessageBoxImage.Error);
            }
            finally
            {
                SetProcessing(false);
            }
        }

        private void Cancel_Click(object sender, RoutedEventArgs e)
        {
            if (!_isProcessing)
            {
                DialogResult = false;
                Close();
            }
        }

        private void SetProcessing(bool processing)
        {
            _isProcessing = processing;
            SendButton.IsEnabled = !processing;
            CancelButton.IsEnabled = !processing;
            AddressText.IsEnabled = !processing;
            AmountText.IsEnabled = !processing;
            ProcessingPanel.Visibility = processing ? Visibility.Visible : Visibility.Collapsed;
        }
    }
}
