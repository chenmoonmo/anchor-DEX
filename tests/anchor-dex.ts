import * as web3 from "@solana/web3.js";
import * as token from "@solana/spl-token";
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorDex } from "../target/types/anchor_dex";
import { assert } from "chai";

interface Pool {
  auth: web3.Keypair;
  payer: web3.Keypair;
  mint0: web3.PublicKey;
  mint1: web3.PublicKey;
  vault0: web3.PublicKey;
  vault1: web3.PublicKey;
  poolMint: web3.PublicKey;
  poolState: web3.PublicKey;
  poolAuth: web3.PublicKey;
}

interface LPProvider {
  signer: web3.Keypair;
  user0: web3.PublicKey;
  user1: web3.PublicKey;
  poolAta: web3.PublicKey;
}

describe("anchor-dex", () => {
  let provider = anchor.AnchorProvider.env();
  let connection = provider.connection;
  anchor.setProvider(provider);

  const program = anchor.workspace.AnchorDex as Program<AnchorDex>;

  let pool: Pool;
  let n_decimals = 9;

  it("initializes a new pool", async () => {
    let auth = web3.Keypair.generate();
    const sig = await connection.requestAirdrop(
      auth.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    const latestBlockHash = await connection.getLatestBlockhash();

    await connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: sig,
    });

    let mint0 = await token.createMint(
      connection,
      auth,
      auth.publicKey,
      auth.publicKey,
      n_decimals
    );
    let mint1 = await token.createMint(
      connection,
      auth,
      auth.publicKey,
      auth.publicKey,
      n_decimals
    );
    // publickey and bump
    let [poolState, poolState_b] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("pool_state"), mint0.toBuffer(), mint1.toBuffer()],
      program.programId
    );

    let [authority, authority_b] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("authority"), poolState.toBuffer()],
      program.programId
    );
    let [vault0, vault0_b] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault0"), poolState.toBuffer()],
      program.programId
    );
    let [vault1, vault1_b] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault1"), poolState.toBuffer()],
      program.programId
    );
    let [poolMint, poolMint_b] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("pool_mint"), poolState.toBuffer()],
      program.programId
    );

    await program.methods
      .initializePool()
      .accounts({
        mint0: mint0,
        mint1: mint1,
        poolAuthority: authority,
        vault0: vault0,
        vault1: vault1,
        poolMint: poolMint,
        poolState: poolState,
        payer: provider.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: token.TOKEN_PROGRAM_ID,
      })
      .rpc();

    pool = {
      auth: auth,
      payer: auth,
      mint0: mint0,
      mint1: mint1,
      vault0: vault0,
      vault1: vault1,
      poolMint: poolMint,
      poolState: poolState,
      poolAuth: authority,
    };

    const res = await program.account.poolState.all();

    const b0 = await connection.getTokenAccountBalance(pool.vault0);

    console.log(b0);

    console.log(res, res[0].account.token0Mint.equals(mint0));
  });

  // helper function
  async function setup_lp_provider(lp_user: web3.PublicKey, amount: number) {
    // setup token accs for deposit
    let mint0_ata = await token.createAssociatedTokenAccount(
      connection,
      pool.payer,
      pool.mint0,
      lp_user
    );
    let mint1_ata = await token.createAssociatedTokenAccount(
      connection,
      pool.payer,
      pool.mint1,
      lp_user
    );

    // setup token accs for LP pool tokens
    let pool_mint_ata = await token.createAssociatedTokenAccount(
      connection,
      pool.payer,
      pool.poolMint,
      lp_user
    );

    // setup initial balance of mints
    await token.mintTo(
      connection,
      pool.payer,
      pool.mint0,
      mint0_ata,
      pool.auth,
      amount * 10 ** n_decimals
    );
    await token.mintTo(
      connection,
      pool.payer,
      pool.mint1,
      mint1_ata,
      pool.auth,
      amount * 10 ** n_decimals
    );

    return [mint0_ata, mint1_ata, pool_mint_ata];
  }

  async function get_token_balance(pk) {
    return (await connection.getTokenAccountBalance(pk)).value.uiAmount;
  }

  function lp_amount(n) {
    return new anchor.BN(n * 10 ** n_decimals);
  }

  let lp_user0: LPProvider;
  it("adds init liquidity to the pool", async () => {
    let lp_user_signer = web3.Keypair.generate();
    let lp_user = lp_user_signer.publicKey;

    let [user0, user1, poolAta] = await setup_lp_provider(lp_user, 100);

    lp_user0 = {
      signer: lp_user_signer,
      user0: user0,
      user1: user1,
      poolAta: poolAta,
    };

    let [src_amount0_in, src_amount1_in] = [lp_amount(50), lp_amount(50)];

    await program.methods
      .addLiquidity(src_amount0_in, src_amount1_in)
      .accounts({
        poolState: pool.poolState,
        poolAuthority: pool.poolAuth,
        vault0: pool.vault0,
        vault1: pool.vault1,
        poolMint: pool.poolMint,
        user0: user0,
        user1: user1,
        userPoolAta: poolAta,
        owner: lp_user,
        tokenProgram: token.TOKEN_PROGRAM_ID,
      })
      .signers([lp_user_signer])
      .rpc();
    // 用户获得的lp
    let balance_mint0 = await get_token_balance(poolAta);
    let poolState = await program.account.poolState.fetch(pool.poolState);
    // 总的lp
    let amountTotalMint = poolState.totalAmountMinted
      .div(new anchor.BN(10 ** n_decimals))
      .toNumber();
    console.log("user0 lp amout: ", balance_mint0);
    console.log("total mint lp amount", amountTotalMint);
    assert.equal(balance_mint0, amountTotalMint);

    let vb0 = await get_token_balance(pool.vault0);
    let vb1 = await get_token_balance(pool.vault1);
    console.log("vault0 balance: ", vb0);
    console.log("vault1 balance: ", vb1);

    assert.equal(vb0, 50);
    assert.equal(vb1, 50);
  });

  let lp_user1: LPProvider;
  it("adds 2nd liquidity to the pool", async () => {
    let lp_user_signer = web3.Keypair.generate();
    let lp_user = lp_user_signer.publicKey;
    let [user0, user1, poolAta] = await setup_lp_provider(lp_user, 100);

    lp_user1 = {
      signer: lp_user_signer,
      user0: user0,
      user1: user1,
      poolAta: poolAta,
    };

    let [src_amount0_in, src_amount1_in] = [lp_amount(50), lp_amount(50)];

    await program.methods
      .addLiquidity(src_amount0_in, src_amount1_in)
      .accounts({
        poolState: pool.poolState,
        poolAuthority: pool.poolAuth,
        vault0: pool.vault0,
        vault1: pool.vault1,
        poolMint: pool.poolMint,
        user0: user0,
        user1: user1,
        userPoolAta: poolAta,
        owner: lp_user,
        tokenProgram: token.TOKEN_PROGRAM_ID,
      })
      .signers([lp_user_signer])
      .rpc();

    // 用户获得的lp
    let balance_mint0 = await get_token_balance(poolAta);
    let poolState = await program.account.poolState.fetch(pool.poolState);
    // 总的lp
    let amountTotalMint = poolState.totalAmountMinted
      .div(new anchor.BN(10 ** n_decimals))
      .toNumber();
    console.log("user1 lp amout: ", balance_mint0);
    console.log("total mint lp amount", amountTotalMint);

    assert.equal(balance_mint0, 50);
    assert.equal(amountTotalMint, 100);

    let vb0 = await get_token_balance(pool.vault0);
    let vb1 = await get_token_balance(pool.vault1);
    console.log("vault0 balance: ", vb0);
    console.log("vault1 balance: ", vb1);

    assert.equal(vb0, 100);
    assert.equal(vb1, 100);
  });

  it("removes liquidity from the pool", async () => {
    let balance_token0_before = await get_token_balance(lp_user0.user0);
    let balance_token1_before = await get_token_balance(lp_user0.user1);
    let balance_mint0_before = await get_token_balance(lp_user0.poolAta);
    console.log("user0 lp amout: ", balance_mint0_before);
    console.log("balance token0 before: ", balance_token0_before);
    console.log("balance token1 before: ", balance_token1_before);

    await program.methods
      .removeLiquidity(lp_amount(25))
      .accounts({
        poolState: pool.poolState,
        poolAuthority: pool.poolAuth,
        vault0: pool.vault0,
        vault1: pool.vault1,
        poolMint: pool.poolMint,
        userPoolAta: lp_user0.poolAta,
        user0: lp_user0.user0,
        user1: lp_user0.user1,
        owner: lp_user0.signer.publicKey,
        tokenProgram: token.TOKEN_PROGRAM_ID,
      })
      .signers([lp_user0.signer])
      .rpc();

    let balance_mint0_after = await get_token_balance(lp_user0.poolAta);
    console.log("user0 lp amout: ", balance_mint0_after);

    let balance_token0 = await get_token_balance(lp_user0.user0);
    let balance_token1 = await get_token_balance(lp_user0.user1);

    console.log("balance_token0: ", balance_token0);
    console.log("balance_token1: ", balance_token1);

    assert.equal(balance_mint0_after, 25);
    assert.equal(balance_token0, 75);
    assert.equal(balance_token1, 75);
  });

  it("swaps token0 for token1", async () => {
    let swap_user = web3.Keypair.generate();
    let swap_user_pk = swap_user.publicKey;

    let user0 = await token.createAssociatedTokenAccount(
      connection,
      pool.payer,
      pool.mint0,
      swap_user_pk
    );

    let user1 = await token.createAssociatedTokenAccount(
      connection,
      pool.payer,
      pool.mint1,
      swap_user_pk
    );

    await token.mintTo(
      connection,
      pool.payer,
      pool.mint0,
      user0,
      pool.auth,
      100 * 10 ** n_decimals
    );

    const user0_balance_before = await get_token_balance(user0);
    const user1_balance_before = await get_token_balance(user1);

    console.log("user0 balance before: ", user0_balance_before);
    console.log("user1 balance before: ", user1_balance_before);

    await program.methods
      .swap(new anchor.BN(10 * 10 ** n_decimals), new anchor.BN(0))
      .accounts({
        poolState: pool.poolState,
        poolAuthority: pool.poolAuth,
        userIn: user0,
        userOut: user1,
        vaultIn: pool.vault0,
        vaultOut: pool.vault1,
        owner: swap_user_pk,
        tokenProgram: token.TOKEN_PROGRAM_ID,
      })
      .signers([swap_user])
      .rpc();

    const user0_balance_after = await get_token_balance(user0);
    const user1_balance_after = await get_token_balance(user1);

    console.log("user0 balance after: ", user0_balance_after);
    console.log("user1 balance after: ", user1_balance_after);

    // k = 100 * 100
    // 100 - k / (100 + 10) =
    assert.equal(user0_balance_after, 90);
    assert.equal(user1_balance_after, +(75 - (75 * 75) / (75 + 10)).toFixed(9));
  });
});
