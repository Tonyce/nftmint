import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";

import {
  TOKEN_PROGRAM_ID,
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import {
  PROGRAM_ID as METADATA_PROGRAM_ID,
  Metadata,
} from "@metaplex-foundation/mpl-token-metadata";

// import { Prog } from '@metaplex-foundation/js-next'
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";

import { Nftmint } from "../target/types/nftmint";

type User = {
  key: anchor.web3.Keypair;
  wallet: anchor.Wallet;
  provider: anchor.Provider;
};
var MPL_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

describe("sol998", () => {
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Nftmint as Program<Nftmint>;

  it("mint hero", async () => {
    const owner = await createUser();
    // const { mint, mta } = await mintHero(owner);
    const name = "test";
    const symbol = "test-symbol";
    const uri = "hero://body";

    const [mint, mintBump] = await PublicKey.findProgramAddress(
      [
        Buffer.from("hero_mint_seed"),
        program.programId.toBuffer(),
        Buffer.from(name),
        Buffer.from(symbol),
        Buffer.from(uri),
      ],
      program.programId
    );
    // console.log({ mint });

    const [mintTokenAccount, mintTokenAccountNump] =
      await PublicKey.findProgramAddress(
        [
          Buffer.from("hero_mint_token_account_seed"),
          program.programId.toBuffer(),
          Buffer.from(name),
          Buffer.from(symbol),
          Buffer.from(uri),
        ],
        program.programId
      );

    // const mintTokenAccount = await getOrCreateAssociatedTokenAccount(
    //   anchor.getProvider().connection,
    //   owner.key,
    //   mint,
    //   owner.key.publicKey
    // );

    // console.log({ mint, mintTokenAccount });

    const [metadataAccount, metadataBump] = await PublicKey.findProgramAddress(
      [
        Buffer.from("metadata"),
        METADATA_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
      ],
      METADATA_PROGRAM_ID
    );

    const r = await program.methods
      .heroMint(name, symbol, uri)
      .accounts({
        heroMetadataAccount: metadataAccount,
        heroMint: mint,
        heroTokenAccount: mintTokenAccount,
        user: owner.key.publicKey,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        mplProgram: MPL_PROGRAM_ID,
      })
      .signers([owner.key])
      .rpc();
    console.log("mintHero result", r);

    const heroMetadata = await Metadata.fromAccountAddress(
      provider.connection,
      metadataAccount
    );
    console.log(heroMetadata);
  });

  async function createUser(airdropBalance?: number): Promise<{
    key: anchor.web3.Keypair;
    wallet: anchor.Wallet;
    provider: anchor.Provider;
  }> {
    airdropBalance = airdropBalance ?? 10 * LAMPORTS_PER_SOL;

    let user = anchor.web3.Keypair.generate();

    let sig = await provider.connection.requestAirdrop(
      user.publicKey,
      airdropBalance
    );

    const result = await provider.connection.confirmTransaction(
      sig,
      "processed"
    );

    // const balance = await getAccountBalance(user.publicKey);

    let wallet = new anchor.Wallet(user);
    let userProvider = new anchor.Provider(
      provider.connection,
      wallet,
      provider.opts
    );

    return {
      key: user,
      wallet,
      provider: userProvider,
    };
  }

  async function mintHero(user: User) {
    const mint = await createMint(
      anchor.getProvider().connection,
      user.key,
      user.key.publicKey,
      user.key.publicKey,
      0
    );

    const mintTokenAccount = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      user.key,
      mint,
      user.key.publicKey
    );

    await mintTo(
      anchor.getProvider().connection,
      user.key,
      mint,
      mintTokenAccount.address,
      user.key,
      1,
      []
    );
    return { mint, mta: mintTokenAccount };
  }
});
