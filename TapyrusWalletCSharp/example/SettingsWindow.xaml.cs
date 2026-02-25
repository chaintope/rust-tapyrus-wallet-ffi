using System;
using System.IO;
using System.Windows;

namespace TapyrusWalletExample
{
    public partial class SettingsWindow : Window
    {
        private static readonly string SettingsPath =
            Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "settings.json");

        public SettingsWindow()
        {
            InitializeComponent();
            LoadSettings();
        }

        private void LoadSettings()
        {
            var settings = AppSettings.Load(SettingsPath);

            if (settings.ConnectionType == ConnectionType.Electrum)
            {
                ElectrumRadio.IsChecked = true;
                EsploraRadio.IsChecked = false;
            }
            else
            {
                EsploraRadio.IsChecked = true;
                ElectrumRadio.IsChecked = false;
            }

            EsploraHostText.Text = settings.EsploraHost;
            EsploraPortText.Text = settings.EsploraPort;
            ElectrumHostText.Text = settings.ElectrumHost;
            ElectrumPortText.Text = settings.ElectrumPort;

            UpdateVisibility();
        }

        private void ConnectionType_Changed(object sender, RoutedEventArgs e)
        {
            UpdateVisibility();
        }

        private void UpdateVisibility()
        {
            if (EsploraGroup == null || ElectrumGroup == null) return;

            if (ElectrumRadio.IsChecked == true)
            {
                EsploraGroup.Visibility = Visibility.Collapsed;
                ElectrumGroup.Visibility = Visibility.Visible;
            }
            else
            {
                EsploraGroup.Visibility = Visibility.Visible;
                ElectrumGroup.Visibility = Visibility.Collapsed;
            }
        }

        private void Apply_Click(object sender, RoutedEventArgs e)
        {
            ValidationError.Text = "";

            var isElectrum = ElectrumRadio.IsChecked == true;

            if (isElectrum)
            {
                if (string.IsNullOrWhiteSpace(ElectrumHostText.Text))
                {
                    ValidationError.Text = "Electrum host is required.";
                    return;
                }
                if (!IsValidPort(ElectrumPortText.Text))
                {
                    ValidationError.Text = "Electrum port must be a number between 0 and 65535.";
                    return;
                }
            }
            else
            {
                if (string.IsNullOrWhiteSpace(EsploraHostText.Text))
                {
                    ValidationError.Text = "Esplora host is required.";
                    return;
                }
                if (!IsValidPort(EsploraPortText.Text))
                {
                    ValidationError.Text = "Esplora port must be a number between 0 and 65535.";
                    return;
                }
            }

            var settings = new AppSettings
            {
                ConnectionType = isElectrum ? ConnectionType.Electrum : ConnectionType.Esplora,
                EsploraHost = EsploraHostText.Text.Trim(),
                EsploraPort = EsploraPortText.Text.Trim(),
                ElectrumHost = ElectrumHostText.Text.Trim(),
                ElectrumPort = ElectrumPortText.Text.Trim()
            };

            settings.Save(SettingsPath);
            DialogResult = true;
            Close();
        }

        private static bool IsValidPort(string portStr)
        {
            return ushort.TryParse(portStr, out _);
        }
    }
}
