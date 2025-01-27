const { Connection, PublicKey, Keypair, SystemProgram, Transaction, TransactionInstruction, sendAndConfirmTransaction } = require('@solana/web3.js');
const bs58 = require('bs58');

const connection = new Connection('https://api.devnet.solana.com', 'confirmed');

const programId = new PublicKey('48xM5iS12LDJj57Dj5rpFC8rEZgWjEpbF7CCwF3rabzs');

const user = Keypair.fromSecretKey(bs58.decode('')); //enter private key

const [userPDA, bumpSeed] = PublicKey.findProgramAddressSync(
    [Buffer.from('user'), user.publicKey.toBuffer()],
    programId
);

async function initializeAccount() {
    const instruction = new TransactionInstruction({
        keys: [
            { pubkey: user.publicKey, isSigner: true, isWritable: true },
            { pubkey: userPDA, isSigner: false, isWritable: true },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        programId: programId,
        data: Buffer.from([0])
    });

    const transaction = new Transaction().add(instruction);

    try {
        const signature = await sendAndConfirmTransaction(connection, transaction, [user]);
        console.log('Initialize account signature:', signature);
        return signature;
    } catch (error) {
        console.error('Error initializing account:', error);
        throw error;
    }
}

async function deposit(amount) {
    const amountBuffer = Buffer.alloc(8);
    amountBuffer.writeBigUInt64LE(BigInt(amount));

    const instruction = new TransactionInstruction({
        keys: [
            { pubkey: user.publicKey, isSigner: true, isWritable: true },
            { pubkey: userPDA, isSigner: false, isWritable: true },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        programId: programId,
        data: Buffer.concat([Buffer.from([1]), amountBuffer])
    });

    const transaction = new Transaction().add(instruction);

    try {
        const signature = await sendAndConfirmTransaction(connection, transaction, [user]);
        console.log('Deposit signature:', signature);
        return signature;
    } catch (error) {
        console.error('Error depositing:', error);
        throw error;
    }
}

async function withdraw(amount) {
    const amountBuffer = Buffer.alloc(8);
    amountBuffer.writeBigUInt64LE(BigInt(amount));

    const instruction = new TransactionInstruction({
        keys: [
            { pubkey: user.publicKey, isSigner: true, isWritable: true },
            { pubkey: userPDA, isSigner: false, isWritable: true },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        programId: programId,
        data: Buffer.concat([Buffer.from([2]), amountBuffer])
    });

    const transaction = new Transaction().add(instruction);

    try {
        const signature = await sendAndConfirmTransaction(connection, transaction, [user]);
        console.log('Withdraw signature:', signature);
        return signature;
    } catch (error) {
        console.error('Error withdrawing:', error);
        throw error;
    }
}

async function getPDABalance() {
    const balance = await connection.getBalance(userPDA);
    console.log('PDA balance:', balance / 1e9, 'SOL');
    return balance;
}

async function main() {
    try {
        console.log('User public key:', user.publicKey.toBase58());
        console.log('PDA address:', userPDA.toBase58());

        await initializeAccount();

        await getPDABalance();

        await deposit(100000000);
        await getPDABalance();

        await withdraw(50000000);
        await getPDABalance();

    } catch (error) {
        console.error('Error in main:', error);
    }
}

main();