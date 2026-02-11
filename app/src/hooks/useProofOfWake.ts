import { useMemo } from 'react';
import * as anchor from '@coral-xyz/anchor';
import { useMobileWallet } from '@wallet-ui/react-native-kit';
import { Connection, PublicKey } from '@solana/web3.js';
import idl from '../constants/idl.json';

const PROGRAM_ID = new PublicKey('3XY5vp1p4Q9fCeCwQNz3yMikYZhoXFJDDmEp6dBXMpx4');

export function useProofOfWake() {
  const { account, authorizeSession } = useMobileWallet();

  const program = useMemo(() => {
    const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
    
    // Create a dummy wallet if not connected
    const mockWallet = {
      publicKey: account ? new PublicKey(account.address) : PublicKey.default,
      signTransaction: async (tx: any) => {
        const { transientSession } = await authorizeSession();
        return transientSession.signTransactions([tx])[0];
      },
      signAllTransactions: async (txs: any[]) => {
        const { transientSession } = await authorizeSession();
        return transientSession.signTransactions(txs);
      },
    } as anchor.Wallet;

    const provider = new anchor.AnchorProvider(connection, mockWallet, {
      preflightCommitment: 'confirmed',
    });

    return new anchor.Program(idl as anchor.Idl, provider);
  }, [account]);

  const getChallengeAddress = (user: PublicKey) => {
    return PublicKey.findProgramAddressSync(
      [Buffer.from('challenge'), user.toBuffer()],
      PROGRAM_ID
    )[0];
  };

  const getTreasuryAddress = () => {
    return PublicKey.findProgramAddressSync(
      [Buffer.from('treasury')],
      PROGRAM_ID
    )[0];
  };

  return {
    program,
    getChallengeAddress,
    getTreasuryAddress,
    account,
  };
}
