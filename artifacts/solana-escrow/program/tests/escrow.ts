/**
 * Anchor test suite for the Solana Escrow program.
 *
 * Run with: anchor test (from the program directory with Anchor CLI installed)
 *
 * These tests cover:
 * 1. Creating an escrow and locking SOL
 * 2. Releasing funds to the seller
 * 3. Cancelling an escrow and refunding the buyer
 */

import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Escrow } from "../target/types/escrow";
import {
  PublicKey,
  SystemProgram,
  LAMPORTS_PER_SOL,
  Keypair,
} from "@solana/web3.js";
import { assert } from "chai";

describe("escrow", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Escrow as Program<Escrow>;
  const buyer = provider.wallet as anchor.Wallet;

  let seller: Keypair;
  let escrowPDA: PublicKey;
  let escrowBump: number;
  let escrowId: BN;

  before(async () => {
    seller = Keypair.generate();
    escrowId = new BN(Date.now());

    [escrowPDA, escrowBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        buyer.publicKey.toBuffer(),
        escrowId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    // Airdrop SOL to seller for rent exemption
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(seller.publicKey, 0.5 * LAMPORTS_PER_SOL),
      "confirmed"
    );
  });

  it("Creates an escrow and locks SOL in PDA", async () => {
    const amountLamports = new BN(0.1 * LAMPORTS_PER_SOL);
    const buyerBalanceBefore = await provider.connection.getBalance(buyer.publicKey);

    await program.methods
      .createEscrow(seller.publicKey, amountLamports, escrowId)
      .accounts({
        buyer: buyer.publicKey,
        escrowAccount: escrowPDA,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const escrowAccount = await program.account.escrowAccount.fetch(escrowPDA);

    assert.equal(escrowAccount.buyer.toBase58(), buyer.publicKey.toBase58(), "Buyer should match");
    assert.equal(escrowAccount.seller.toBase58(), seller.publicKey.toBase58(), "Seller should match");
    assert.equal(escrowAccount.amount.toNumber(), amountLamports.toNumber(), "Amount should match");
    assert.deepEqual(escrowAccount.status, { pending: {} }, "Status should be Pending");
    assert.equal(escrowAccount.bump, escrowBump, "Bump should match");

    const pdaBalance = await provider.connection.getBalance(escrowPDA);
    assert.isAtLeast(pdaBalance, amountLamports.toNumber(), "PDA should hold the locked SOL");

    console.log(`✓ Escrow created. PDA: ${escrowPDA.toBase58()}`);
    console.log(`  Locked: ${amountLamports.toNumber() / LAMPORTS_PER_SOL} SOL`);
  });

  it("Releases funds to the seller", async () => {
    const sellerBalanceBefore = await provider.connection.getBalance(seller.publicKey);

    await program.methods
      .releaseFunds()
      .accounts({
        buyer: buyer.publicKey,
        seller: seller.publicKey,
        escrowAccount: escrowPDA,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const escrowAccount = await program.account.escrowAccount.fetch(escrowPDA);
    assert.deepEqual(escrowAccount.status, { completed: {} }, "Status should be Completed");

    const sellerBalanceAfter = await provider.connection.getBalance(seller.publicKey);
    assert.isAbove(sellerBalanceAfter, sellerBalanceBefore, "Seller should have received SOL");

    console.log(
      `✓ Funds released. Seller received: ${(sellerBalanceAfter - sellerBalanceBefore) / LAMPORTS_PER_SOL} SOL`
    );
  });

  it("Creates a second escrow and cancels it (refunds buyer)", async () => {
    const escrowId2 = new BN(Date.now() + 1);
    const [escrowPDA2] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        buyer.publicKey.toBuffer(),
        escrowId2.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    const amountLamports = new BN(0.05 * LAMPORTS_PER_SOL);

    await program.methods
      .createEscrow(seller.publicKey, amountLamports, escrowId2)
      .accounts({
        buyer: buyer.publicKey,
        escrowAccount: escrowPDA2,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const buyerBalanceBefore = await provider.connection.getBalance(buyer.publicKey);

    await program.methods
      .cancelEscrow()
      .accounts({
        buyer: buyer.publicKey,
        escrowAccount: escrowPDA2,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const escrowAccount = await program.account.escrowAccount.fetch(escrowPDA2);
    assert.deepEqual(escrowAccount.status, { cancelled: {} }, "Status should be Cancelled");

    const buyerBalanceAfter = await provider.connection.getBalance(buyer.publicKey);
    assert.isAbove(buyerBalanceAfter, buyerBalanceBefore, "Buyer should have received refund");

    console.log(`✓ Escrow cancelled. Buyer refunded: ${(buyerBalanceAfter - buyerBalanceBefore) / LAMPORTS_PER_SOL} SOL`);
  });

  it("Prevents non-buyer from releasing funds", async () => {
    const escrowId3 = new BN(Date.now() + 2);
    const [escrowPDA3] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        buyer.publicKey.toBuffer(),
        escrowId3.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    const amountLamports = new BN(0.05 * LAMPORTS_PER_SOL);

    await program.methods
      .createEscrow(seller.publicKey, amountLamports, escrowId3)
      .accounts({
        buyer: buyer.publicKey,
        escrowAccount: escrowPDA3,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    try {
      await program.methods
        .releaseFunds()
        .accounts({
          buyer: seller.publicKey,
          seller: seller.publicKey,
          escrowAccount: escrowPDA3,
          systemProgram: SystemProgram.programId,
        })
        .signers([seller])
        .rpc();
      assert.fail("Should have thrown an error");
    } catch (err: unknown) {
      const msg = (err as Error).message;
      assert.include(msg, "NotBuyer", "Should reject non-buyer with NotBuyer error");
      console.log("✓ Non-buyer correctly rejected");
    }
  });
});
