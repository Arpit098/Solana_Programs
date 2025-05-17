// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { Marketplace } from "../target/types/marketplace";
// import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
// import { 
//   TOKEN_2022_PROGRAM_ID, 
//   createMint, 
//   getAssociatedTokenAddress,
//   ASSOCIATED_TOKEN_PROGRAM_ID,
//   createAssociatedTokenAccount,
//   mintTo
// } from "@solana/spl-token";

// describe("nft_marketplace", () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);
//   const program = anchor.workspace.Marketplace as Program<Marketplace>;
//   const wallet = provider.wallet as anchor.Wallet;
//   const buyer = Keypair.generate();

//   let tokenMint: PublicKey;
//   let makerTokenAccount: PublicKey;
//   let auction: PublicKey;
//   let vault: PublicKey;

//   before(async () => {
//     // Airdrop SOL
//     await provider.connection.confirmTransaction(
//       await provider.connection.requestAirdrop(
//         wallet.publicKey,
//         1000000000 // 1 SOL
//       )
//     );
//     await provider.connection.confirmTransaction(
//       await provider.connection.requestAirdrop(
//         buyer.publicKey,
//         1000000000 // 1 SOL
//       )
//     );

//     tokenMint = await createMint(
//       provider.connection,
//       wallet.payer,
//       wallet.publicKey,
//       null,
//       0, // 0 decimals for NFT
//       undefined,
//       undefined,
//       TOKEN_2022_PROGRAM_ID
//     );

//     makerTokenAccount = await getAssociatedTokenAddress(
//       tokenMint,
//       wallet.publicKey,
//       false,
//       TOKEN_2022_PROGRAM_ID,
//       ASSOCIATED_TOKEN_PROGRAM_ID
//     );

//     await createAssociatedTokenAccount(
//       provider.connection,
//       wallet.payer,
//       tokenMint,
//       wallet.publicKey,
//       undefined,
//       TOKEN_2022_PROGRAM_ID,
//       ASSOCIATED_TOKEN_PROGRAM_ID
//     );

//     // Mint 1 NFT token
//     await mintTo(
//       provider.connection,
//       wallet.payer,
//       tokenMint,
//       makerTokenAccount,
//       wallet.publicKey,
//       1, // Mint 1 NFT
//       [],
//       undefined,
//       TOKEN_2022_PROGRAM_ID
//     );
//   });

//   it("Create auction", async () => {
//     const [derivedAuction] = PublicKey.findProgramAddressSync(
//       [
//         tokenMint.toBuffer(),
//         wallet.publicKey.toBuffer()
//       ],
//       program.programId
//     );

//     auction = derivedAuction;
    
//     vault = await getAssociatedTokenAddress(
//       tokenMint,
//       auction,
//       true,
//       TOKEN_2022_PROGRAM_ID,
//       ASSOCIATED_TOKEN_PROGRAM_ID
//     );
    
//     try {
//       const tx = await program.methods
//         .createAuction(
//           "Test NFT",
//           new anchor.BN(1000000), // Price
//           new anchor.BN(Date.now()/1000 + 86400), // 24hr validity
//           wallet.publicKey
//         )
//         .accounts({
//           owner: wallet.publicKey,
//           tokenMint: tokenMint,
//           makerTokenAccount: makerTokenAccount,
//           auction: auction,
//           vault: vault,
//           systemProgram: SystemProgram.programId,
//           tokenProgram: TOKEN_2022_PROGRAM_ID,
//           associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
//         })
//         .rpc();

//       const auctionAccount = await program.account.auction.fetch(auction);
//       console.log("Auction created:", auctionAccount);
//     } catch (error) {
//       console.error("Error creating auction:", error);
//       throw error;
//     }
//   });

//   it("Buy token", async () => {

//     const buyerTokenMint = await createMint(
//       provider.connection,
//       buyer,
//       buyer.publicKey,
//       null,
//       6,
//       undefined,
//       undefined,
//       TOKEN_2022_PROGRAM_ID
//     );

//     // Create buyer's payment token account
//     const buyerTokenAccountMoney = await getAssociatedTokenAddress(
//       buyerTokenMint,
//       buyer.publicKey,
//       false,
//       TOKEN_2022_PROGRAM_ID,
//       ASSOCIATED_TOKEN_PROGRAM_ID
//     );

//     await createAssociatedTokenAccount(
//       provider.connection,
//       buyer,
//       buyerTokenMint,
//       buyer.publicKey,
//       undefined,
//       TOKEN_2022_PROGRAM_ID,
//       ASSOCIATED_TOKEN_PROGRAM_ID
//     );

//     // Mint payment tokens to buyer
//     await mintTo(
//       provider.connection,
//       buyer,
//       buyerTokenMint,
//       buyerTokenAccountMoney,
//       buyer.publicKey,
//       2000000, // More than auction price
//       [],
//       undefined,
//       TOKEN_2022_PROGRAM_ID
//     );

//     // Create buyer's NFT token account
//     const buyerTokenAccountItem = await getAssociatedTokenAddress(
//       tokenMint,
//       buyer.publicKey,
//       false,
//       TOKEN_2022_PROGRAM_ID,
//       ASSOCIATED_TOKEN_PROGRAM_ID
//     );

//     await createAssociatedTokenAccount(
//       provider.connection,
//       buyer,
//       tokenMint,
//       buyer.publicKey,
//       undefined,
//       TOKEN_2022_PROGRAM_ID,
//       ASSOCIATED_TOKEN_PROGRAM_ID
//     );

//     // Create seller's payment token account
//     const ownerTokenReceiveAccount = await getAssociatedTokenAddress(
//       buyerTokenMint,
//       wallet.publicKey,
//       false,
//       TOKEN_2022_PROGRAM_ID,
//       ASSOCIATED_TOKEN_PROGRAM_ID
//     );

//     await createAssociatedTokenAccount(
//       provider.connection,
//       buyer,
//       buyerTokenMint,
//       wallet.publicKey,
//       undefined,
//       TOKEN_2022_PROGRAM_ID,
//       ASSOCIATED_TOKEN_PROGRAM_ID
//     );

//     try {
//       const tx = await program.methods
//         .buyToken()
//         .accounts({
//           buyer: buyer.publicKey,
//           owner: wallet.publicKey,
//           tokenMint: tokenMint,
//           buyerTokenMint: buyerTokenMint,
//           buyerTokenAccountMoney: buyerTokenAccountMoney,
//           buyerTokenAccountItem: buyerTokenAccountItem,
//           ownerTokenReceiveAccount: ownerTokenReceiveAccount,
//           auction: auction,
//           vault: vault,
//           systemProgram: SystemProgram.programId,
//           tokenProgram: TOKEN_2022_PROGRAM_ID,
//           associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//         })
//         .signers([buyer])
//         .rpc();

//       const auctionAccount = await program.account.auction.fetch(auction);
//       console.log("Updated auction state:", auctionAccount);
//     } catch (error) {
//       console.error("Error buying token:", error);
//       throw error;
//     }
//   });

// //   it("Recreate auction", async () => {
// //     const [newAuction] = PublicKey.findProgramAddressSync(
// //       [
// //         buyer.publicKey.toBuffer(),
// //         tokenMint.toBuffer()
// //       ],
// //       program.programId
// //     );

// //     const newVault = await getAssociatedTokenAddress(
// //       tokenMint,
// //       newAuction,
// //       true,
// //       TOKEN_2022_PROGRAM_ID,
// //       ASSOCIATED_TOKEN_PROGRAM_ID
// //     );

// //     try {
// //       const tx = await program.methods
// //         .recreateAuction(
// //           new anchor.BN(1500000), // New price
// //           buyer.publicKey
// //         )
// //         .accounts({
// //           maker: buyer.publicKey,
// //           tokenMint: tokenMint,
// //           originalCreator: wallet.publicKey,
// //           originalAuction: auction,
// //           makerTokenAccount: await getAssociatedTokenAddress(
// //             tokenMint,
// //             buyer.publicKey,
// //             false,
// //             TOKEN_2022_PROGRAM_ID,
// //             ASSOCIATED_TOKEN_PROGRAM_ID
// //           ),
// //           auction: newAuction,
// //           vault: newVault,
// //           systemProgram: SystemProgram.programId,
// //           tokenProgram: TOKEN_2022_PROGRAM_ID,
// //           associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
// //         })
// //         .signers([buyer])
// //         .rpc();

// //       const newAuctionAccount = await program.account.auction.fetch(newAuction);
// //       console.log("New auction created:", newAuctionAccount);
// //     } catch (error) {
// //       console.error("Error recreating auction:", error);
// //       throw error;
// //     }
// //   });
// });