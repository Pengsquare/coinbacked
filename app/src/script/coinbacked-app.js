var coinbackedApp = coinbackedApp || {};

coinbackedApp = 
{
    connected: false,
    publicKey: null,
    mintAccounts: [],
    tokenAccounts: [],
    backedAccounts: [],

    runInstructions: function(instructions, success = (signature)=>{}, failure = (error)=>{})
    {
        let connection = new solanaWeb3.Connection(solanaWeb3.clusterApiUrl('devnet'), 'confirmed');

        var transaction = new solanaWeb3.Transaction();
        instructions.forEach((instruction) => transaction.add(instruction));

        connection.getRecentBlockhash().then((hash) => 
        {
          transaction.recentBlockhash = hash.blockhash;
          transaction.feePayer = coinbackedApp.publicKey;

          window.solana.signTransaction(transaction).then((signedTransaction) => 
          {
            connection.sendRawTransaction(signedTransaction.serialize()).then((signature) => 
            {
              success(signature);
            });
          });
        });
    },

    resetData: function()
    {
        // reset globals
        coinbackedApp.mintAccounts = [];
        coinbackedApp.tokenAccounts = [];
        coinbackedApp.backedAccounts = [];

        // reset UI
        $('#app-box-list').empty();
        $('#app-box-info-mint-count').text('-');
        $('#app-box-info-nft-count').text('-');
        $('#app-box-info-backed-count').text('-');
        $('#app-box-info-backed-value').text('-');
    },

    resetApp: function()
    {
        coinbackedApp.resetData();
    },

    loadTokenListData: function(provider)
    {
        // load all tokens...
        let mintsToFetch = 0;
        let nftCounter = 0;
        let backedCounter = 0;
        let backedLamports = 0n;

        let connection = new solanaWeb3.Connection(solanaWeb3.clusterApiUrl('devnet'), 'confirmed');
        let api = new coinbackedWeb3.api(connection);
         
        connection.getTokenAccountsByOwner(coinbackedApp.publicKey, {programId: new solanaWeb3.PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA')}).then((result) =>
        {
            mintsToFetch = result.value.length;
            result.value.forEach(element => 
            {
                let tokenAccount = 
                {
                    mint:  new solanaWeb3.PublicKey(element.account.data.slice(0,32)),
                    owner: new solanaWeb3.PublicKey(element.account.data.slice(32,64)),
                    amount: element.account.data.slice(64,72).readBigUInt64LE(0)
                };

                coinbackedApp.tokenAccounts.push(tokenAccount);

                // get mint account
                connection.getAccountInfo(tokenAccount.mint).then(result=>
                {
                    let mintAccount = 
                    {
                        key: tokenAccount.mint,
                        supply: result.data.slice(36,44).readBigUInt64LE(0),  
                        isSupplyFixed: (new solanaWeb3.PublicKey(result.data.slice(4,36)) == null), 
                        decimals: result.data.slice(44, 45).readUint8(0)
                    }

                    if (mintAccount.supply == 1)
                    {
                        nftCounter++;
                    }

                    coinbackedApp.mintAccounts.push(mintAccount);
                    mintsToFetch--;

                    if (mintsToFetch == 0)
                    {
                        $('#app-box-info-mint-count').text(coinbackedApp.mintAccounts.length);
                        $('#app-box-info-nft-count').text(nftCounter);

                        coinbackedApp.mintAccounts.forEach(element =>
                        {
                            let associtatedAccount = coinbackedApp.tokenAccounts.find(token => token.mint == element.key );

                            $('#app-box-list').append(
                                "<div class='card' style='margin: 20px' id='entry_" + element.key +"'>" +
                                    "<div class='row'>" + 
                                        "<div class='col-4 token-info-col'>" + 
                                            "<header><h4>" +"<span style='font-weight: bold' id='entry_title_" + element.key +"'>Unknown Token</span> (" + coinbackedApp.shortenPublicKeyString(element.key) + ")</h4></header>"+
                                            api.tokenAmountUI(associtatedAccount.amount, element.decimals)  +
                                        "</div>" + 
                                        "<div class='col-2 token-info-col is-hidden backing-info'>" +
                                            "Your Value <span id='backed_value_token_" + element.key + "'>0.00</span>" +
                                        "</div>" +
                                        "<div class='col-2 token-info-col is-hidden backing-info'>" +
                                            "Total Value  <span id='backed_value_total_" + element.key + "'>0.00</span>" +
                                        "</div>" +
                                        "<div class='col'>" +
                                            "<div id='btn_start_backing_" + element.key + "' class='btn-start-backing button primary outline pull-right non-backed is-hidden'>" +
                                                "Start backing" +
                                            "</div>" +
                                            "<div class='button primary outline backed pull-right is-hidden'>" +
                                                "Burn" +
                                            "</div>" +
                                            "<div id='btn_validate_" + element.key + "' class='btn-validate button primary outline backed pull-right is-hidden'>" +
                                                "Validate" +
                                            "</div>" +
                                            "<div class='button primary outline backed pull-right is-hidden'>" +
                                                "Add Funds" +
                                            "</div>" +
                                        "</div>" +
                                    "</div>" +
                                "</div>" 
                            );
                        });


                        $('.btn-start-backing').on('click', (e)=>
                        {
                            coinbackedApp.togglePopup('#popup-start-backing-info');
                        });

                        $('.btn-validate').on('click', (e)=>
                        {
                            coinbackedApp.togglePopup('#popup-validate-info', ()=>
                            {
                                let mintKey = new solanaWeb3.PublicKey(e.target.id.split("_")[2]);
                                
                                let connection = new solanaWeb3.Connection(solanaWeb3.clusterApiUrl('devnet'), 'confirmed');
                                let api = new coinbackedWeb3.api(connection);
                                let instructionsAPI = new coinbackedWeb3.instructions(api);

                                instructionsAPI.validationInstructions(mintKey, coinbackedApp.publicKey).then((instructions)=>
                                {
                                    coinbackedApp.runInstructions(instructions, (signature) =>
                                    {
                                        $('#popup-validation-results-logs').text(signature);
                                        coinbackedApp.togglePopup('#popup-validation-results');
                                    });
                                });
                                
                            });
                        });

                        $('#hero-intro').hide();
                        $('#concept').hide();
                        $('#info-block').hide();


                        coinbackedApp.togglePopup('#popup-alpha');
                        $('.pageloader').removeClass('is-active');

                    }

                    api.isMintBacked(mintAccount.key).then((result)=>
                    {
                        if (result)
                        {
                            // update global stuff
                            coinbackedApp.backedAccounts.push(result);
                            backedCounter++;
                            $('#app-box-info-backed-count').text(backedCounter);

                             // toggle label
                             $('#entry_' + mintAccount.key).find('.backing-info').removeClass('is-hidden');
                             $('#entry_' + mintAccount.key).find('.backed').removeClass('is-hidden');

                            let token = coinbackedApp.tokenAccounts.find(token => token.mint == mintAccount.key );
                            api.getPayoutInLamports(mintAccount.key, token.amount).then(lamports =>
                            {
                                // entry
                                $('#backed_value_token_' + mintAccount.key.toBase58()).text("◎" + api.solanaAmountUI(lamports,5));

                                // overview
                                backedLamports += lamports;
                                $('#app-box-info-backed-value').text("◎" + api.solanaAmountUI(backedLamports,5));
                                $('.pageloader').removeClass('is-active');

                            });

                            api.getBackingLamports(mintAccount.key).then(lamports =>
                            {
                                $('#backed_value_total_' + mintAccount.key.toBase58()).text("◎" + api.solanaAmountUI(lamports,5));
                            });

                           
                        }
                        else
                        {
                            $('#entry_' + mintAccount.key).find('.non-backed').removeClass('is-hidden');
                        }
                    });
                });

            });


        });

    },

    eventButtonConnectClicked: function()
    {
        $('#pageloader-text').text("Connecting to Wallet...");
        $('.pageloader').addClass('is-active');

        let provider = coinbackedApp.getProvider();
        if (provider.isConnected)
        {
            // refresh only
            coinbackedApp.resetData();
            coinbackedApp.loadTokenListData(provider);
            return;
        }

        coinbackedApp.resetApp();

        if (provider)
        {
            provider.on('connect', () =>
            {   
                coinbackedApp.publicKey = provider.publicKey;
                $('#btn-connect').text("↻ " + provider.publicKey.toBase58().slice(0,12)+'...');
                $('#btn-connect').addClass('primary outline');
                $('#btn-connect').removeClass('success');

                coinbackedApp.loadTokenListData(provider);

            });
        }
        provider.connect();
        
        $('#app-box').removeClass('is-hidden');
    },

    getProvider: function()
    {
        if ("phantom" in window) 
        {
            const provider = window.phantom.solana;
            if (provider.isPhantom) 
            {
                return provider;
            }
        }
        window.open("https://phantom.app/", "_blank");
    },

    setup: function()
    {
        $('#btn-connect').on('click', ()=> 
        {
            coinbackedApp.eventButtonConnectClicked();
        });
    },

    shortenPublicKeyString: function(key)
    {
        let base = key.toBase58();
        return(base.substr(0, 4) + '...' + base.substr(base.length - 4));
    },

    togglePopup: function(id, nextBlock = ()=>{})
    {
        if ($(id).hasClass("is-hidden"))
        {
            $(id).find('.popup-okay').on("click", ()=>{ coinbackedApp.togglePopup(id)});
            $(id).find('.popup-continue').on("click", ()=>
            { 
                coinbackedApp.togglePopup(id); 
                nextBlock();
            });

            $("body").css('overflow', 'hidden');
            $(id).removeClass("is-hidden");
        }
        else
        {
            $(id).find('.popup-okay').off();
            $(id).find('.popup-continue').off();

            $("body").css('overflow', 'auto');
            $(id).addClass("is-hidden")
        }
    }
};