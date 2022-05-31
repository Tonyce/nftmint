import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Metadata } from "@metaplex-foundation/mpl-token-metadata";

import { Nftmint } from "../target/types/nftmint";

import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createMint,
} from "@solana/spl-token";

const { web3 } = anchor;

// const MPL_PROGRAM_ID = new web3.PublicKey(
//   "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
// );
// const METADATA_PROGRAM_ID = new PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s')
// const MPL_PROGRAM_ID = new PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s')

type User = {
  key: anchor.web3.Keypair;
  wallet: anchor.Wallet;
  provider: anchor.Provider;
};

describe("nftmint", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Nftmint as Program<Nftmint>;

  // it.skip("Does CPI", async () => {
  //   const puppetKeypair = web3.Keypair.generate();
  //   const r = await program.methods
  //     .test()
  //     .accounts({
  //       puppet: puppetKeypair.publicKey,
  //       user: anchor.getProvider().wallet.publicKey,
  //       systemProgram: web3.SystemProgram.programId,
  //     })
  //     .signers([puppetKeypair])
  //     .rpc();
  //   console.log({ r });
  // });

  it("Is initialized!", async () => {
    const user = await createUser();

    const name = "testName";
    const symbol = "testSymbol";
    const uri =
      "https://ipfs.io/ipfs/Qmb2ZL1Csp8Kdtdvcx8mKXmr9rLeko5KT1FS8BYKEYcadw";

    const [mint] = await web3.PublicKey.findProgramAddress(
      [Buffer.from("nft-mint-seed")],
      program.programId
    );
    // const mint = await createMint(
    //   anchor.getProvider().connection,
    //   user.key,
    //   user.key.publicKey,
    //   user.key.publicKey,
    //   0
    // );
    const nftTokenAccount = await getAssociatedTokenAddress(
      mint,
      user.key.publicKey
    );

    console.log(mint.toString(), nftTokenAccount.toString());

    // const nftMetadataPDA = await getMetadataPDAFromMint(nftMintAccount);

    // console.log({ nftMintAccount: nftMintAccount.toString() });
    const result = await program.methods
      .mintWithTokenaccount(name, symbol, uri)
      .accounts({
        heroMint: mint,
        heroTokenAccount: nftTokenAccount,
        user: user.key.publicKey,
        systemProgram: web3.SystemProgram.programId,
        rent: web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([user.key])
      .rpc();

    console.log("Your transaction result", result);

    // const accountInfo = await anchor
    //   .getProvider()
    //   .connection.getAccountInfo(nftMetadataPDA);
    // console.log("accountInfo", accountInfo);

    // const data = await Metadata.fromAccountAddress(
    //   anchor.getProvider().connection,
    //   nftMetadataPDA
    // );
    // console.log(data);
  });

  async function createUser(airdropBalance?: number): Promise<User> {
    airdropBalance = airdropBalance ?? 10 * web3.LAMPORTS_PER_SOL;

    let user = anchor.web3.Keypair.generate();

    let sig = await provider.connection.requestAirdrop(
      user.publicKey,
      airdropBalance
    );

    const result = await provider.connection.confirmTransaction(
      sig,
      "processed"
    );

    const balance = await getAccountBalance(user.publicKey);
    console.log({ balance });
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

  async function getAccountBalance(pubkey) {
    let account = await provider.connection.getAccountInfo(pubkey);
    return account?.lamports ?? 0;
  }
});
