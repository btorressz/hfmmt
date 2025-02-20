
describe("HFMM Token Program", () => {
  // Define the SPL Token Program ID as a constant.
  const TOKEN_PROGRAM_ID = new anchor.web3.PublicKey(
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
  );

  it("initializes the config", async () => {
    const configKp = new web3.Keypair();
    const rewardRate = new BN(100);
    const volatilityThreshold = new BN(50);
    // Pass a single object with the expected keys
    const txHash = await pg.program.methods
      .initialize({ rewardRate, volatilityThreshold })
      .accounts({
        config: configKp.publicKey,
        user: pg.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([configKp])
      .rpc();
    console.log(`Config initialized. Tx: ${txHash}`);

    const configAccount = await pg.program.account.config.fetch(configKp.publicKey);
    // Assert that on-chain values match our inputs.
    assert(configAccount.rewardRate.eq(rewardRate));
    assert(configAccount.volatilityThreshold.eq(volatilityThreshold));
  });

  it("stakes tokens", async () => {
    const marketMakerKp = new web3.Keypair();
    // Replace with valid public keys from your test environment.
    const stakerTokenAccount = new web3.PublicKey("StakerTokenAccountPublicKey");
    const vaultTokenAccount = new web3.PublicKey("VaultTokenAccountPublicKey");
    const stakeAmount = new BN(500);
    const txHash = await pg.program.methods
      .stakeTokens(stakeAmount)
      .accounts({
        staker: marketMakerKp.publicKey,
        stakerTokenAccount: stakerTokenAccount,
        vault: vaultTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([marketMakerKp])
      .rpc();
    console.log(`Stake tokens. Tx: ${txHash}`);

    const marketMakerAccount = await pg.program.account.marketMaker.fetch(
      marketMakerKp.publicKey
    );
    assert(marketMakerAccount.stakedAmount.eq(stakeAmount));
  });

  it("updates performance and distributes rewards", async () => {
    const marketMakerKp = new web3.Keypair();
    const execVolume = new BN(200);
    const spreadEfficiency = new BN(100);
    const orderFlow = new BN(50);

    let txHash = await pg.program.methods
      .updatePerformance(execVolume, spreadEfficiency, orderFlow)
      .accounts({
        staker: marketMakerKp.publicKey,
      })
      .signers([marketMakerKp])
      .rpc();
    console.log(`Update performance. Tx: ${txHash}`);

    txHash = await pg.program.methods
      .distributeRewards()
      .accounts({
        staker: marketMakerKp.publicKey,
        vault: new web3.PublicKey("VaultTokenAccountPublicKey"),
        stakerTokenAccount: new web3.PublicKey("StakerTokenAccountPublicKey"),
        config: new web3.PublicKey("ConfigAccountPublicKey"),
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([marketMakerKp])
      .rpc();
    console.log(`Distribute rewards. Tx: ${txHash}`);
  });

  it("provides and withdraws liquidity", async () => {
    const liquidityProviderKp = new web3.Keypair();
    const liquidityAmount = new BN(1000);
    const providerTokenAccount = new web3.PublicKey("ProviderTokenAccountPublicKey");
    const poolTokenAccount = new web3.PublicKey("PoolTokenAccountPublicKey");

    let txHash = await pg.program.methods
      .provideLiquidity(liquidityAmount)
      .accounts({
        liquidityProvider: liquidityProviderKp.publicKey,
        providerTokenAccount: providerTokenAccount,
        pool: poolTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([liquidityProviderKp])
      .rpc();
    console.log(`Provide liquidity. Tx: ${txHash}`);

    // Withdraw half of the liquidity.
    txHash = await pg.program.methods
      .withdrawLiquidity(liquidityAmount.div(new BN(2)))
      .accounts({
        liquidityProvider: liquidityProviderKp.publicKey,
        pool: poolTokenAccount,
        userTokenAccount: new web3.PublicKey("UserTokenAccountPublicKey"),
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([liquidityProviderKp])
      .rpc();
    console.log(`Withdraw liquidity. Tx: ${txHash}`);
  });

  it("claims fee rebate", async () => {
    const marketMakerKp = new web3.Keypair();
    const feeRebatePoolKp = new web3.Keypair();
    const stakerTokenAccount = new web3.PublicKey("StakerTokenAccountPublicKey");
    const txHash = await pg.program.methods
      .claimFeeRebate()
      .accounts({
        staker: marketMakerKp.publicKey,
        feeRebatePool: feeRebatePoolKp.publicKey,
        stakerTokenAccount: stakerTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([marketMakerKp])
      .rpc();
    console.log(`Claim fee rebate. Tx: ${txHash}`);
  });

  it("locks liquidity", async () => {
    const liquidityProviderKp = new web3.Keypair();
    const lockDuration = new BN(3); // 3-month lock.
    const txHash = await pg.program.methods
      .lockLiquidity(lockDuration)
      .accounts({
        liquidityProvider: liquidityProviderKp.publicKey,
      })
      .signers([liquidityProviderKp])
      .rpc();
    console.log(`Lock liquidity. Tx: ${txHash}`);
  });

  it("enables auto compound", async () => {
    const marketMakerKp = new web3.Keypair();
    const txHash = await pg.program.methods
      .enableAutoCompound()
      .accounts({
        staker: marketMakerKp.publicKey,
      })
      .signers([marketMakerKp])
      .rpc();
    console.log(`Enable auto compound. Tx: ${txHash}`);
  });

  it("claims liquidity rewards", async () => {
    const liquidityProviderKp = new web3.Keypair();
    const poolTokenAccount = new web3.PublicKey("PoolTokenAccountPublicKey");
    const providerTokenAccount = new web3.PublicKey("ProviderTokenAccountPublicKey");
    const txHash = await pg.program.methods
      .claimLiquidityRewards()
      .accounts({
        liquidityProvider: liquidityProviderKp.publicKey,
        pool: poolTokenAccount,
        providerTokenAccount: providerTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([liquidityProviderKp])
      .rpc();
    console.log(`Claim liquidity rewards. Tx: ${txHash}`);
  });

  it("enforces activity slashing", async () => {
    const marketMakerKp = new web3.Keypair();
    const treasuryTokenAccount = new web3.PublicKey("TreasuryTokenAccountPublicKey");
    const txHash = await pg.program.methods
      .enforceActivitySlashing()
      .accounts({
        staker: marketMakerKp.publicKey,
        treasury: treasuryTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([marketMakerKp])
      .rpc();
    console.log(`Enforce activity slashing. Tx: ${txHash}`);
  });

  it("enforces minimum trade volume", async () => {
    const marketMakerKp = new web3.Keypair();
    const txHash = await pg.program.methods
      .enforceMinTradeVolume()
      .accounts({
        staker: marketMakerKp.publicKey,
      })
      .signers([marketMakerKp])
      .rpc();
    console.log(`Enforce min trade volume. Tx: ${txHash}`);
  });

  it("enforces stake cooldown", async () => {
    const marketMakerKp = new web3.Keypair();
    const txHash = await pg.program.methods
      .enforceStakeCooldown()
      .accounts({
        staker: marketMakerKp.publicKey,
      })
      .signers([marketMakerKp])
      .rpc();
    console.log(`Enforce stake cooldown. Tx: ${txHash}`);
  });

  it("adjusts rewards by volatility", async () => {
    const marketMakerKp = new web3.Keypair();
    const volatilityIndex = new BN(60); // High volatility.
    const txHash = await pg.program.methods
      .adjustRewardsByVolatility(volatilityIndex)
      .accounts({
        staker: marketMakerKp.publicKey,
      })
      .signers([marketMakerKp])
      .rpc();
    console.log(`Adjust rewards by volatility. Tx: ${txHash}`);
  });

  it("claims gas fee rebate", async () => {
    const marketMakerKp = new web3.Keypair();
    const gasFeePoolKp = new web3.Keypair();
    const stakerTokenAccount = new web3.PublicKey("StakerTokenAccountPublicKey");
    const txHash = await pg.program.methods
      .claimGasFeeRebate()
      .accounts({
        staker: marketMakerKp.publicKey,
        stakerTokenAccount: stakerTokenAccount,
        gasFeePool: gasFeePoolKp.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([marketMakerKp])
      .rpc();
    console.log(`Claim gas fee rebate. Tx: ${txHash}`);
  });

  it("claims insurance payout", async () => {
    const liquidityProviderKp = new web3.Keypair();
    const insurancePoolKp = new web3.Keypair();
    const providerTokenAccount = new web3.PublicKey("ProviderTokenAccountPublicKey");
    const txHash = await pg.program.methods
      .claimInsurancePayout()
      .accounts({
        liquidityProvider: liquidityProviderKp.publicKey,
        insurancePool: insurancePoolKp.publicKey,
        providerTokenAccount: providerTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([liquidityProviderKp])
      .rpc();
    console.log(`Claim insurance payout. Tx: ${txHash}`);
  });
});
