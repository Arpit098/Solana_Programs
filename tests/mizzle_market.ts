// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { Demo } from "../target/types/demo";
// import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
// import { 
//   TOKEN_2022_PROGRAM_ID, 
//   createMint, 
//   getAssociatedTokenAddress,
//   ASSOCIATED_TOKEN_PROGRAM_ID,
//   createAssociatedTokenAccount,
//   mintTo
// } from "@solana/spl-token";

// describe("mizzle_market", () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);
//   const program = anchor.workspace.Demo as Program<Demo>;
//   const wallet = provider.wallet as anchor.Wallet;
//   const buyer = Keypair.generate();

//   let tokenMint: PublicKey;
//   let makerTokenAccount: PublicKey;
//   let auction: PublicKey;
//   let vault: PublicKey;
//   let newAuction: PublicKey;
//   let newVault : PublicKey;
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
//           buyer.publicKey,
//           1000000000 // 1 SOL
//       )
//     );


//     // Create token mint
//     tokenMint = await createMint(
//       provider.connection,
//       wallet.payer,
//       wallet.publicKey, // mint authority
//       null, // freeze authority
//       6, // decimals
//       undefined,
//       undefined,
//       TOKEN_2022_PROGRAM_ID
//     );

//     // Create maker's token account as regular ATA
//     makerTokenAccount = await getAssociatedTokenAddress(
//       tokenMint,
//       wallet.publicKey,
//       false,
//       TOKEN_2022_PROGRAM_ID,
//       ASSOCIATED_TOKEN_PROGRAM_ID
//     );

//     // Create the token account
//     await createAssociatedTokenAccount(
//       provider.connection,
//       wallet.payer,
//       tokenMint,
//       wallet.publicKey,
//       undefined,
//       TOKEN_2022_PROGRAM_ID,
//       ASSOCIATED_TOKEN_PROGRAM_ID
//     );

//     // Mint some tokens to maker's account
//     await mintTo(
//       provider.connection,
//       wallet.payer,
//       tokenMint,
//       makerTokenAccount,
//       wallet.publicKey,
//       1000000, // Amount to mint
//       [],
//       undefined,
//       TOKEN_2022_PROGRAM_ID
//     );
//   });
  
//   it("Create auction", async () => {
//     const serialNum = new anchor.BN(100);
    
//     // Log the seeds for debugging
//     // console.log("Seeds being used:");
//     // console.log("Serial num:", serialNum.toArrayLike(Buffer, "le", 8));
//     // console.log("Token mint:", tokenMint.toBuffer());
//     // console.log("Wallet:", wallet.publicKey.toBuffer());
//     // console.log("PublicKey:", wallet.publicKey)
//     const [derivedAuction] = PublicKey.findProgramAddressSync(
//         [
//             serialNum.toArrayLike(Buffer, "le", 8),
//             tokenMint.toBuffer(),
//             wallet.publicKey.toBuffer()
//         ],
//         program.programId
//     );

//     auction = derivedAuction;
    
//     // Log derived address
//     console.log("Derived auction address:", auction.toBase58());

//     vault = await getAssociatedTokenAddress(
//       tokenMint,
//       auction,
//       true, // allowOwnerOffCurve needs to be true since auction is a PDA
//       TOKEN_2022_PROGRAM_ID,
//       ASSOCIATED_TOKEN_PROGRAM_ID
//     );
    
//     try {
//         const tx = await program.methods
//             .createAuction(
//                 serialNum,
//                 "GPU",
//                 new anchor.BN(1000),
//                 new anchor.BN(5),
//                 new anchor.BN(86400),
//                 wallet.publicKey
//             )
//             .accounts({
//                 owner: wallet.publicKey,
//                 tokenMint: tokenMint,
//                 makerTokenAccount: makerTokenAccount,
//                 auction: auction,
//                 vault: vault,
//                 systemProgram: SystemProgram.programId,
//                 tokenProgram: TOKEN_2022_PROGRAM_ID,
//                 associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
//             });

//         // Get all accounts that will be used
//         const keys = await tx.pubkeys();
//         console.log("Account pubkeys:", keys);

//         await tx.rpc();
        
//         const auctionAccount = await program.account.auction.fetch(auction);
//         console.log("Auction created:", auctionAccount);

//         const auctions = await program.account.auction.all([
           
//           ]);
//           console.log("All auctions for creator:", auctions);
//     } catch (error) {
//         console.error("Error creating auction:", error);
//         // Log more details about the error
//         if (error.logs) {
//             console.log("Program logs:", error.logs);
//         }
//         throw error;
//     }
// });
// it("Buy tokens and then resell", async () => {
//   const serialNum = new anchor.BN(1);

//   const amount = new anchor.BN(100);

//   // Derive auction PDA using same seeds from create_auction
//   const [derivedAuction] = PublicKey.findProgramAddressSync(
//       [
//           serialNum.toArrayLike(Buffer, "le", 8),
//           tokenMint.toBuffer(),
//           wallet.publicKey.toBuffer()
//       ],
//       program.programId
//   );

//   // Verify auction address matches
//   // console.log("Current auction address:", auction.toBase58());
//   // console.log("Derived auction address:", derivedAuction.toBase58());

//   // Create buyer token mint
//   let buyerTokenMint = await createMint(
//       provider.connection,
//       buyer,
//       buyer.publicKey,
//       null,
//       6,
//       undefined,
//       undefined,
//       TOKEN_2022_PROGRAM_ID
//   );

//   // Create buyer's payment token account
//   const buyerTokenAccountMoney = await getAssociatedTokenAddress(
//       buyerTokenMint,
//       buyer.publicKey,
//       false,
//       TOKEN_2022_PROGRAM_ID,
//       ASSOCIATED_TOKEN_PROGRAM_ID
//   );

//   // Create account and mint tokens
//   await createAssociatedTokenAccount(
//       provider.connection,
//       buyer,
//       buyerTokenMint,
//       buyer.publicKey,
//       undefined,
//       TOKEN_2022_PROGRAM_ID,
//       ASSOCIATED_TOKEN_PROGRAM_ID
//   );

//   await mintTo(
//       provider.connection,
//       buyer,
//       buyerTokenMint,
//       buyerTokenAccountMoney,
//       buyer.publicKey,
//       1000000,
//       [],
//       undefined,
//       TOKEN_2022_PROGRAM_ID
//   );

//   // Create buyer's token account for receiving items
//   const buyerTokenAccountItem = await getAssociatedTokenAddress(
//       tokenMint,
//       buyer.publicKey,
//       false,
//       TOKEN_2022_PROGRAM_ID,
//       ASSOCIATED_TOKEN_PROGRAM_ID
//   );

//   await createAssociatedTokenAccount(
//       provider.connection,
//       buyer,
//       tokenMint,
//       buyer.publicKey,
//       undefined,
//       TOKEN_2022_PROGRAM_ID,
//       ASSOCIATED_TOKEN_PROGRAM_ID
//   );

//   // Create owner's token account for receiving payment
//   const ownerTokenReceiveAccount = await getAssociatedTokenAddress(
//       buyerTokenMint,
//       wallet.publicKey,
//       false,
//       TOKEN_2022_PROGRAM_ID,
//       ASSOCIATED_TOKEN_PROGRAM_ID
//   );

//   try {
//       await createAssociatedTokenAccount(
//           provider.connection,
//           buyer,
//           buyerTokenMint,
//           wallet.publicKey,
//           undefined,
//           TOKEN_2022_PROGRAM_ID,
//           ASSOCIATED_TOKEN_PROGRAM_ID
//       );
//   } catch (e) {
//       console.log("Owner's token account already exists");
//   }

//   try {
//       const tx = await program.methods
//           .buyToken(amount)
//           .accounts({
//               buyer: buyer.publicKey,
//               owner: wallet.publicKey,
//               tokenMint: tokenMint,
//               buyerTokenMint: buyerTokenMint,
//               buyerTokenAccountMoney: buyerTokenAccountMoney,
//               buyerTokenAccountItem: buyerTokenAccountItem,
//               ownerTokenReceiveAccount: ownerTokenReceiveAccount,
//               auction: derivedAuction, // Use derived PDA that matches create_auction
//               vault: vault,
//               systemProgram: SystemProgram.programId,
//               tokenProgram: TOKEN_2022_PROGRAM_ID,
//               associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//           })
//           .signers([buyer]);

//       console.log("Transaction accounts:", await tx.pubkeys());
//       await tx.rpc();
      
//       const auctionAccount = await program.account.auction.fetch(derivedAuction);
//       console.log("Updated auction state:", auctionAccount);   
  
//   } catch (error) {
//       console.error("Error buying tokens:", error);
//       if (error.logs) {
//           console.log("Program logs:", error.logs);
//       }
//       throw error;
//   }



// });
// });
//   try {
//     const [originalAuction] = PublicKey.findProgramAddressSync(
//       [
//           serialNum.toArrayLike(Buffer, "le", 8),
//           tokenMint.toBuffer(),
//           wallet.publicKey.toBuffer() // Original creator
//       ],
//       program.programId
//   );
//     const [newAuction] = PublicKey.findProgramAddressSync(
//             [
//                 serialNum.toArrayLike(Buffer, "le", 8),
//                 buyer.publicKey.toBuffer(),
//                 tokenMint.toBuffer()
//             ],
//             program.programId
//         );
      
//     const newVault = await getAssociatedTokenAddress(
//             tokenMint,
//             newAuction,
//             true,
//             TOKEN_2022_PROGRAM_ID,
//             ASSOCIATED_TOKEN_PROGRAM_ID
//      );
//     const tx = await program.methods
//         .recreateAuction(
//             serialNum,
//             "GPU123", // Same GPU string as original
//             new anchor.BN(50),
//             new anchor.BN(7),
//             new anchor.BN(86400),
//             buyer.publicKey
//         )
//         .accounts({
//             maker: buyer.publicKey,
//             tokenMint: tokenMint,
//             originalAuction: auction, // Original auction for validation
//             makerTokenAccount: buyerTokenAccountItem,
//             auction: newAuction,
//             vault: newVault,
//             systemProgram: SystemProgram.programId,
//             tokenProgram: TOKEN_2022_PROGRAM_ID,
//             associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//         })
//         .signers([buyer]);

//     console.log("Transaction accounts:", await tx.pubkeys());
//     await tx.rpc();
    
//     // Verify the new auction state
//     const newAuctionAccount = await program.account.auction.fetch(newAuction);
//     console.log("Buyer's resale auction created:", newAuctionAccount);

//     // Verify original auction still exists
//     const originalAuctionAccount = await program.account.auction.fetch(originalAuction);
//     console.log("Original auction state:", originalAuctionAccount);

// } catch (error) {
//     console.error("Error recreating auction:", error);
//     if (error.logs) {
//         console.log("Program logs:", error.logs);
//     }
//     throw error;
// }