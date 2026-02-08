import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ProofOfWake } from "../target/types/proof_of_wake";
import { expect } from "chai";

describe("proof-of-wake", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.ProofOfWake as Program<ProofOfWake>;
  const provider = anchor.getProvider();
  const authority = (provider as anchor.AnchorProvider).wallet.publicKey;

  const [treasuryPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("treasury")],
    program.programId
  );

  const [challengePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("challenge"), authority.toBuffer()],
    program.programId
  );

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initializeTreasury().accounts({
      treasury: treasuryPda,
      authority: authority,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).rpc();
    console.log("Your transaction signature", tx);

    const treasuryAccount = await program.account.treasury.fetch(treasuryPda);
    expect(treasuryAccount.authority.toBase58()).to.equal(authority.toBase58());
  });

  it("Starts a challenge!", async () => {
    const tx = await program.methods.startChallenge(7, 0).accounts({
      challenge: challengePda,
      authority: authority,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).rpc();
    
    const challengeAccount = await program.account.challenge.fetch(challengePda);
    expect(challengeAccount.streak).to.equal(0);
    expect(challengeAccount.isActive).to.be.true;
    expect(challengeAccount.alarmHour).to.equal(7);
  });

  it("Completes a day!", async () => {
    // For testing window logic, we might need to use a localnet and skip time
    const tx = await program.methods.completeDay().accounts({
      challenge: challengePda,
      treasury: treasuryPda,
      authority: authority,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).rpc();

    const challengeAccount = await program.account.challenge.fetch(challengePda);
    expect(challengeAccount.streak).to.equal(1);
  });
});
