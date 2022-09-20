import * as anchor from "@project-serum/anchor";
import { Program, BN } from "@project-serum/anchor";
import { TokenContract } from "../target/types/token_contract";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  transfer,
  getAccount,
  createAccount,
} from "@solana/spl-token";
import { clusterApiUrl,
    Connection,
    Keypair,
    LAMPORTS_PER_SOL
} from '@solana/web3.js';
import { assert } from "chai";
import BigNumber from 'big-number';
import web3 from '@solana/web3.js';

// async function airdropTo() {
//     const payer = Keypair.generate();

//     const connection = new Connection(
//         clusterApiUrl('devnet'),
//         'confirmed'
//     );

//     const airdropSignature = await connection.requestAirdrop(
//         payer.publicKey,
//         LAMPORTS_PER_SOL,
//     );

//     await connection.confirmTransaction(airdropSignature);
// }

describe("SPL token testing", () => {
    // it("Mint a token is zero", async () => {
    //     const payer = Keypair.generate();
    //     const mintAuthority = Keypair.generate();
    //     const freezeAuthority = Keypair.generate();

    //     const connection = new Connection(
    //         clusterApiUrl('devnet'),
    //         'confirmed'
    //     );

    //     const airdropSignature = await connection.requestAirdrop(
    //         payer.publicKey,
    //         LAMPORTS_PER_SOL,
    //     );
          
    //     await connection.confirmTransaction(airdropSignature);

    //     const mint = await createMint(
    //         connection,
    //         payer,
    //         mintAuthority.publicKey,
    //         freezeAuthority.publicKey,
    //         9 // We are using 9 to match the CLI decimal default exactly
    //     );

    //     console.log(mint.toBase58());
        
    //     const mintInfo = await getMint(
    //         connection,
    //         mint
    //     )
        
    //     console.log(mintInfo.supply);

    //     assert.equal(mintInfo.supply, BigNumber(0));
    // });

    it("Transfer a token", async () => {
        const connection = new Connection(clusterApiUrl('devnet'), 'confirmed');

        const wallet = Keypair.generate();
        const auxiliaryKeypair = Keypair.generate();

        const airdropSignature = await connection.requestAirdrop(
            wallet.publicKey,
            LAMPORTS_PER_SOL,
        );

        await connection.confirmTransaction(airdropSignature);

        const mint = await createMint(
            connection,
            wallet,
            wallet.publicKey,
            wallet.publicKey,
            9
        );

        // Create custom token account
        const auxiliaryTokenAccount = await createAccount(
            connection,
            wallet,
            mint,
            wallet.publicKey,
            auxiliaryKeypair
        );

        const associatedTokenAccount = await getOrCreateAssociatedTokenAccount(
            connection,
            wallet,
            mint,
            wallet.publicKey
        );

        await mintTo(
            connection,
            wallet,
            mint,
            associatedTokenAccount.address,
            wallet,
            50
        );

        const accountInfo = await getAccount(connection, associatedTokenAccount.address);

        console.log(accountInfo.amount);
        // 50

        await transfer(
            connection,
            wallet,
            associatedTokenAccount.address,
            auxiliaryTokenAccount,
            wallet,
            50
        );

        const auxAccountInfo = await getAccount(connection, auxiliaryTokenAccount);

        console.log(auxAccountInfo.amount);
        // 50

        await transfer(
            connection,
            wallet,
            auxiliaryTokenAccount,
            associatedTokenAccount.address,
            wallet,
            50
        );

        const backInfo = await getAccount(connection, associatedTokenAccount.address);

        console.log(backInfo.amount);
        // 50
    })
});