import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Blackhat } from "../target/types/blackhat";
import { BN } from "bn.js";
import { keccak_256 } from "js-sha3";
import { publicKey } from "@coral-xyz/anchor/dist/cjs/utils";
import { PublicKey } from "@solana/web3.js";

describe("blackhat", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();

  const program = anchor.workspace.Blackhat as Program<Blackhat>;

  it("Happy path!", async () => {
    // Setup
    const creator = anchor.web3.Keypair.generate();
    const player = anchor.web3.Keypair.generate();
    const max_score = new BN(Math.floor(100 * Math.random()))
    const bet = new BN(1000000000);
    const salt = new BN(Math.floor(Math.random() * 0x100000000)) // random u32
    const buffer = Buffer.concat([
      new anchor.BN(max_score).toArrayLike(Buffer, "le", 8),
      new anchor.BN(salt).toArrayLike(Buffer, "le", 8),
    ]);
    const commitment = Buffer.from(keccak_256(buffer), "hex");
    const [game, _gameBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode("game")),
        player.publicKey.toBuffer()
      ],
      program.programId
    );
    const [gameAuthority, _gameAuthorityBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode("authority")),
        game.toBuffer()
      ],
      program.programId
    );

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        creator.publicKey,
        1000000000 * 10
      )
    );

    const tx = await program.methods
      .setup(bet, commitment.toJSON().data)
      .accounts({
        creator: creator.publicKey,
        player: player.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        game,
        gameAuthority
      })
      .signers([creator])
      .rpc();
    console.log("Your transaction signature", tx);

    // Join
    const r = new BN(Math.floor(Math.random() * 10000))

    await provider.connection.confirmTransaction(await provider.connection.requestAirdrop(
      player.publicKey,
      1000000000 * 10
    )
    );

    const tx2 = await program.methods
      .join(r)
      .accounts({
        player: player.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        game,
        gameAuthority
      })
      .signers([player])
      .rpc();
    console.log("Your transaction signature", tx2);

    // Submit
    const score = new BN(88)
    const tx3 = await program.methods
      .submit(score)
      .accounts({
        player: player.publicKey,
        game,
      })
      .signers([player])
      .rpc();
    console.log("Your transaction signature", tx3);

    // Settle
    console.log("commitment, max_score, salt", commitment.toString("hex"), max_score.toNumber(), salt.toNumber())
    const tx4 = await program.methods
      .settle(max_score, salt, score)
      .accounts({
        creator: creator.publicKey,
        player: player.publicKey,
        game,
        gameAuthority
      })
      .signers([creator])
      .rpc({ skipPreflight: true });
    console.log("Your transaction signature", tx4);
  });

});