import * as anchor from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  getAssociatedTokenAddress,
  mintTo,
  getAccount,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

const program = pg.program;
const connection = pg.connection;
const payer = pg.wallet.keypair;
const user = payer.publicKey;

const DECIMALS = 6;
const UNIT = 1_000_000;

// Create test SPL token
const depositMint = await createMint(
  connection,
  payer,
  user,
  null,
  DECIMALS
);

// Create user token account
const userToken = await getOrCreateAssociatedTokenAccount(
  connection,
  payer,
  depositMint,
  user
);

// Give user 20 test tokens
await mintTo(
  connection,
  payer,
  depositMint,
  userToken.address,
  payer,
  20 * UNIT
);

// Derive program addresses
const [vaultState] = PublicKey.findProgramAddressSync(
  [Buffer.from("vault"), depositMint.toBuffer()],
  program.programId
);

const [shareMint] = PublicKey.findProgramAddressSync(
  [Buffer.from("share_mint"), depositMint.toBuffer()],
  program.programId
);

const vaultTokens = await getAssociatedTokenAddress(
  depositMint,
  vaultState,
  true
);

const userShares = await getAssociatedTokenAddress(
  shareMint,
  user
);

// Initialize vault
await program.methods
  .initialize()
  .accounts({
    user,
    depositMint,
    vaultState,
    shareMint,
    vaultTokens,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .rpc();

console.log("Vault initialized!");

// Deposit 10 tokens
const tx = await program.methods
  .deposit(new anchor.BN(10 * UNIT))
  .accounts({
    user,
    depositMint,
    vaultState,
    shareMint,
    userTokens: userToken.address,
    vaultTokens,
    userShares,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .rpc();

// Check balances
const userBalance = await getAccount(
  connection,
  userToken.address
);

const vaultBalance = await getAccount(
  connection,
  vaultTokens
);

const shareBalance = await getAccount(
  connection,
  userShares
);

console.log("Deposit transaction:", tx);
console.log(
  "User tokens:",
  Number(userBalance.amount) / UNIT
);
console.log(
  "Vault tokens:",
  Number(vaultBalance.amount) / UNIT
);
console.log(
  "Share tokens:",
  Number(shareBalance.amount) / UNIT
);

console.log(
  "Explorer: https://explorer.solana.com/tx/" +
  tx +
  "?cluster=devnet"
);
