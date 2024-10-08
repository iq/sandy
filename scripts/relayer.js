const { Keypair, PublicKey, Connection, VersionedTransaction, TransactionMessage } = require("@solana/web3.js");
const { getAssociatedTokenAddress, createAssociatedTokenAccountIdempotentInstruction } = require('@solana/spl-token');
const fs = require('fs');
const borsh = require('@coral-xyz/borsh');
const { BN } = require('bn.js');
const ws = require('ws');

(async () => {
  const wss = new ws.Server({ port: 8080 });

  wss.on('connection', async (ws) => {
    console.log('Connected to client');

    const serializedTransaction = await sendSwapTransaction();

    const pendingTransaction = {
      transactions: [{
        data: Array.from(serializedTransaction),
        meta: {
          size: serializedTransaction.length,
        }
      }],
    }

    ws.send(JSON.stringify(pendingTransaction));
  });

})();

const sendSwapTransaction = async () => {
  const keypair = Keypair.fromSecretKey(new Uint8Array(JSON.parse(fs.readFileSync('../victim.json'))));

  const connection = new Connection('https://api.mainnet-beta.solana.com', 'confirmed');

  const payer = keypair.publicKey;
  const tokenAddress = new PublicKey('ED5nyyWEzpPPiWimP8vYm7sD7TD3LAt3Q3gRTWHzPJBY')
  const amm = new PublicKey('22WrmyTj8x2TRVQen3fxxi2r4Rn6JDHWoMTpsSmn8RUd')
  const ammAuthority = new PublicKey('5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1')
  const poolCoinTokenAccount = new PublicKey('BqK3MWZbv3DgcsKZBZR1KWVbhKGf2Y5uMdjvqWu4So74')
  const poolPcTokenAccount = new PublicKey('7eoNaWgHkMQrcABabYyvbHzTS3aRyDTRDWVWUARJwfUA')
  const userSourceTokenAccount = await getAssociatedTokenAddress(new PublicKey('So11111111111111111111111111111111111111112'), payer)
  const userDestinationTokenAccount = await getAssociatedTokenAddress(tokenAddress, payer)

  const createTokenAccountInstruction = createAssociatedTokenAccountIdempotentInstruction(
    payer,
    userDestinationTokenAccount,
    payer,
    tokenAddress
  );

  const buffer = Buffer.alloc(100);
  const layout = borsh.struct([borsh.u64('amount_in'), borsh.u64('minimum_amount_out')]);

  const len = layout.encode(
    {
      amount_in: new BN(0.1 * 10 ** 9),
      minimum_amount_out: new BN(0),
    },
    buffer
  );

  const swapInstruction = {
    programId: new PublicKey('675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8'),
    keys: [
      { pubkey: new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'), isSigner: false, isWritable: false },
      { pubkey: amm, isSigner: false, isWritable: true },
      { pubkey: ammAuthority, isSigner: false, isWritable: false },
      { pubkey: amm, isSigner: false, isWritable: true },
      { pubkey: poolCoinTokenAccount, isSigner: false, isWritable: true },
      { pubkey: poolPcTokenAccount, isSigner: false, isWritable: true },
      { pubkey: amm, isSigner: false, isWritable: false },
      { pubkey: amm, isSigner: false, isWritable: false },
      { pubkey: amm, isSigner: false, isWritable: true },
      { pubkey: amm, isSigner: false, isWritable: true },
      { pubkey: amm, isSigner: false, isWritable: true },
      { pubkey: amm, isSigner: false, isWritable: true },
      { pubkey: amm, isSigner: false, isWritable: true },
      { pubkey: amm, isSigner: false, isWritable: false },
      { pubkey: userSourceTokenAccount, isSigner: false, isWritable: true },
      { pubkey: userDestinationTokenAccount, isSigner: false, isWritable: true },
      { pubkey: payer, isSigner: true, isWritable: true },
    ],
    data: Buffer.concat([Buffer.from([9]), buffer]).subarray(0, 1 + len),
  }

  const latestBlockhash = await connection.getLatestBlockhash();

  const messageV0 = new TransactionMessage({
    payerKey: payer,
    recentBlockhash: latestBlockhash.blockhash,
    instructions: [createTokenAccountInstruction, swapInstruction],
  }).compileToV0Message();

  const transaction = new VersionedTransaction(messageV0);
  transaction.sign([keypair]);

  const serializedTransaction = transaction.serialize();

  setTimeout(async () => {
    const signature = await connection.sendRawTransaction(serializedTransaction, {
      skipPreflight: false,
    });

    console.log('Transaction signature:', signature);
  }, 1000);

  return serializedTransaction;
}
