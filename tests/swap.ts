import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getAssociatedTokenAddress,
  mintTo,
  createAssociatedTokenAccount,
  getAccount,
} from "@solana/spl-token";

describe("admin_vault", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Swap as Program<Swap>;
  const admin = provider.wallet as anchor.Wallet;
  const user = Keypair.generate();
console.log(admin.publicKey.toString());
  let mint: PublicKey;
  let vaultPda: PublicKey;
  let vaultBump: number;
  let vaultTokenAccount: PublicKey;
  let adminTokenAccount: PublicKey;
  let userTokenAccount: PublicKey;

  before(async () => {
    // Airdrop SOL to admin and user
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(admin.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL)
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(user.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL)
    );

    // Create mint
    mint = await createMint(
      provider.connection,
      admin.payer,
      admin.publicKey,
      null,
      6 // decimals
    );

    // Derive Vault PDA
    [vaultPda, vaultBump] = await PublicKey.findProgramAddress(
      [Buffer.from("vault"), mint.toBuffer()],
      program.programId
    );

    // Associated token accounts
    vaultTokenAccount = await getAssociatedTokenAddress(
      mint,
      vaultPda,
      true,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    adminTokenAccount = await getAssociatedTokenAddress(
      mint,
      admin.publicKey,
      false,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    userTokenAccount = await getAssociatedTokenAddress(
      mint,
      user.publicKey,
      false,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    // Create admin and user ATAs if needed
    await createAssociatedTokenAccount(
      provider.connection, admin.payer, mint, admin.publicKey
    );
    await createAssociatedTokenAccount(
      provider.connection, admin.payer, mint, user.publicKey
    );
    await createAssociatedTokenAccount(
      provider.connection, admin.payer, mint, vaultPda, true
    );

    // Mint tokens to admin
    await mintTo(
      provider.connection,
      admin.payer,
      mint,
      adminTokenAccount,
      admin.publicKey,
      1_000_000_000
    );
  });

  it("Initializes the vault", async () => {
    await program.methods
      .initialize(new anchor.BN(1_000_000))
      .accounts({
        authority: admin.publicKey,
        vault: vaultPda,
        tokenMint: mint,
        vaultTokenAccount: vaultTokenAccount,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([])
      .rpc();
  });

  it("Updates price", async () => {
    await program.methods
      .updatePrice(new anchor.BN(2_000_000))
      .accounts({
        authority: admin.publicKey,
        vault: vaultPda,
      })
      .signers([])
      .rpc();
  });

  it("Deposits tokens", async () => {
    await program.methods
      .depositTokens(new anchor.BN(100_000_000))
      .accounts({
        authority: admin.publicKey,
        vault: vaultPda,
        tokenMint: mint,
        adminTokenAccount: adminTokenAccount,
        vaultTokenAccount: vaultTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([])
      .rpc();

    const vaultAcct = await getAccount(provider.connection, vaultTokenAccount);
    console.log("Vault token account balance after deposit:", Number(vaultAcct.amount));
  });

  it("User purchases tokens", async () => {
    await program.methods
      .purchaseTokens(new anchor.BN(10_000_000))
      .accounts({
        buyer: user.publicKey,
        admin: admin.publicKey,
        vault: vaultPda,
        tokenMint: mint,
        vaultTokenAccount: vaultTokenAccount,
        userTokenAccount: userTokenAccount,
        vaultSigner: vaultPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    const userAcct = await getAccount(provider.connection, userTokenAccount);
    console.log("User token account balance after purchase:", Number(userAcct.amount));
    const vaultAcct = await getAccount(provider.connection, vaultTokenAccount);
    console.log("Vault token account balance after purchase:", Number(vaultAcct.amount));
  });
});