namespace TapyrusWalletCSharp.Tests;

using com.chaintope.tapyrus.wallet;

public class WalletTest
{
    [Fact]
    public void Test()
    {
        var config = new Config(Network.Prod, 12345, "localhost", 5432, null, null);
        var wallet = new HdWallet(config);

        Assert.NotNull(wallet);

        var address = wallet.GetNewAddress(null);
        Assert.Equal("15Q1z9LJGeaU6oHeEvT1SKoeCUJntZZ9Tg", address);
    }
}