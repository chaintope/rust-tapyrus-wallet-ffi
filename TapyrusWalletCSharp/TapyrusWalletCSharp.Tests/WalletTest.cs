using Xunit.Abstractions;

namespace TapyrusWalletCSharp.Tests;

using System.Runtime.InteropServices;
using com.chaintope.tapyrus.wallet;

public class WalletTest
{
    private readonly ITestOutputHelper _testOutputHelper;

    public WalletTest(ITestOutputHelper testOutputHelper)
    {
        _testOutputHelper = testOutputHelper;
    }

    [Fact]
    public void Test()
    {
        // Create wallet
        var wallet =　GetWallet();

        Assert.NotNull(wallet);

        // Get new address
        var address = wallet.GetNewAddress(null);
        Assert.True(address.Length > 0);
        System.Console.WriteLine(address);

        // Calculate pay to contract address
        var publicKey = "027a1f78d888431b1262f9acf58e625871f161059f8afd43f68f23ba52aef76740";
        var contract = "37a42c03-0fb8-40bd-a4da-2ea9465ae23b";
        var colorId = "c21ea750d7355507cdcc6165679f57945d6593ccf94b0e950b6bc3178ba177a352";
        var contractAddress = wallet.CalcP2cAddress(publicKey, contract, colorId);
        Assert.Equal("15Q1z9LJGeaU6oHeEvT1SKoeCUJntZZ9Tg", contractAddress);

        // Store contract
        var contractId = "contract id";
        var contractRecord = new Contract(contractId, contract, publicKey, false);
        wallet.StoreContract(contractRecord);

        // Update contract
        wallet.UpdateContract(contractId, null, null, true);

        // Get TPC balance
        Assert.Equal((ulong)0, wallet.Balance(null));

        // Get Token balance
        Assert.Equal((ulong)0, wallet.Balance(colorId));

        // Transfer Token
        var toAddress = "1GPpDWt46NxZJCPqMW6LgMqZ1WgkeiyamP";
        var transferParams = new TransferParams(50, toAddress);
        var txid = "7835455e85c69f52a65db4e74cd69b3e99f5db9fbe77fdc20fe60c2f52f3fd4e";
        var index = (uint)0;
        var amount = (ulong)50;
        var unspent = true;
        var txout = new TxOut(txid, index, amount, colorId, address, unspent);
        var utxos = new List<TxOut> { txout };     
        var transferTxid = wallet.Transfer([transferParams], utxos);
        Assert.Equal("2fa3170debe6bdcd98f2ef1fb0dc1368693b5ace4c8eabf549cb6c44616c2819", transferTxid);

        // Get transaction
        var tx = wallet.GetTransaction(transferTxid);
        Assert.Equal("01000000011e86d7726322a1af403815466e44465bd6f119919a20680009b47b4ae00192a5210000006441f09130c3181d20273923f00544e398f4d51315bde28cd4a292d0acda92e9e7ba22c6767c7780828dbf0955add4615f9a2781672ed1afbb8b599a638b20b88ae60121039a77f4e4e45847e413617099b1b4e26d73f372d824432db3c005cabab28c4cccffffffff01d0070000000000001976a914c6e613b40de534b908a283c410f1847943eb629888ac00000000",
            tx);

        // Get transaction Out
        var txOutList = wallet.GetTxOutByAddress(tx, address);
        Assert.Single(txOutList);
        Assert.Equal("2fa3170debe6bdcd98f2ef1fb0dc1368693b5ace4c8eabf549cb6c44616c2819", txOutList[0].txid);
        Assert.Equal((uint)0, txOutList[0].index);
        Assert.Equal((ulong)10, txOutList[0].amount);
        Assert.Null(txOutList[0].colorId);
        Assert.Equal("15Q1z9LJGeaU6oHeEvT1SKoeCUJntZZ9Tg", txOutList[0].address);
        Assert.False(txOutList[0].unspent);
    }

    public HdWallet GetWallet()
    {
        var config = new Config(
            networkMode: Network.Prod,
            networkId: 1939510133,
            genesisHash: "038b114875c2f78f5a2fd7d8549a905f38ea5faee6e29a3d79e547151d6bdd8a",
            esploraHost: "localhost",
            esploraPort: 3001,
            masterKeyPath: "../../../master_key",
            dbFilePath: "wallet.sqlite");
        return new HdWallet(config);
    }

    [Fact]
    public void TestWithEsplora()
    {
        var wallet = GetWallet();
        wallet.FullSync();

        Assert.True(wallet.Balance(null) > 0);
        var address = wallet.GetNewAddress(null);

        // Transfer TPC to faucet
        var transferParams = new TransferParams(1000, address);
        var txid = wallet.Transfer([transferParams], []);
        _testOutputHelper.WriteLine(txid);

        var colorId = "c14ca2241021165f86cf706351de7e235d7f4b4895fcb4d9155a4e9245f95c2c9a";
        var balance = wallet.Balance(colorId);
        Assert.Equal((ulong)100, balance);
    }

    [Fact]
    public void TestColoredCoinWithEsplora()
    {
        var wallet = GetWallet();
        wallet.FullSync();

        Assert.True(wallet.Balance(null) > 2000);

        var colorId = "c14ca2241021165f86cf706351de7e235d7f4b4895fcb4d9155a4e9245f95c2c9a";
        var balance = wallet.Balance(colorId);
        Assert.Equal(balance, (ulong)100);

        var to_address = wallet.GetNewAddress(colorId);
        String txid = wallet.Transfer([new TransferParams(1, to_address)], []);

        // Wait for transaction to be indexed
        String tx = null;
        while (true)
        {
            try
            {
                tx = wallet.GetTransaction(txid);
                break;
            }
            catch (GetTransactionException.UnknownTxid e)
            {
                Thread.Sleep(1000);
            }
        }

        wallet.Sync();

        var txout = wallet.GetTxOutByAddress(tx, to_address);
        var another_address = wallet.GetNewAddress(colorId);
        var txid2 = wallet.Transfer([new TransferParams(1, another_address)], txout);
    }
}