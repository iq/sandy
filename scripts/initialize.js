const { Keypair, PublicKey, TransactionMessage, Connection, VersionedTransaction } = require('@solana/web3.js');
const borsh = require('@coral-xyz/borsh');
const fs = require('fs');
const { BN } = require('bn.js');

// replace with the address of the program
const SANDWICH_PROGRAM = new PublicKey('11111111111111111111111111111111');

(async () => {
  const keypair = Keypair.fromSecretKey(new Uint8Array(JSON.parse(fs.readFileSync('../payer.json'))));

  const payer = keypair.publicKey;
  const sandwichState = PublicKey.findProgramAddressSync([Buffer.from('sandwich-state')], SANDWICH_PROGRAM)[0];
  const systemProgram = new PublicKey('11111111111111111111111111111111');

  const buffer = Buffer.alloc(100);
  const layout = borsh.struct([borsh.u64('preswap_sol_balance'), borsh.u16('tip_bps')]);

  const len = layout.encode(
    {
      preswap_sol_balance: new BN(0),
      // 30%
      tip_bps: new BN(3000),
    },
    buffer
  );

  const initializeInstruction = {
    programId: SANDWICH_PROGRAM,
    keys: [
      { pubkey: payer, isSigner: true, isWritable: false },
      { pubkey: sandwichState, isSigner: false, isWritable: true },
      { pubkey: systemProgram, isSigner: false, isWritable: false },
    ],
    data: Buffer.concat([Buffer.from([0]), buffer]).subarray(0, 1 + len),
  };

  const connection = new Connection('https://api.mainnet-beta.solana.com', 'confirmed');
  const latestBlockhash = await connection.getLatestBlockhash();

  const messageV0 = new TransactionMessage({
    payerKey: payer,
    recentBlockhash: latestBlockhash.blockhash,
    instructions: [initializeInstruction],
  }).compileToV0Message();

  const transaction = new VersionedTransaction(messageV0);
  transaction.sign([keypair]);

  const signature = await connection.sendRawTransaction(transaction.serialize(), {
    skipPreflight: false,
  });

  console.log('Transaction signature:', signature);

})();