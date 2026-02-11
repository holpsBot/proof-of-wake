import React, { useState, useEffect, useCallback } from 'react';
import { View, Text, ScrollView, Pressable, ActivityIndicator, Alert } from 'react-native';
import { Header } from '../components/Header';
import { ChallengeCard } from '../components/ChallengeCard';
import { MathLock } from '../components/MathLock';
import { useProofOfWake } from '../hooks/useProofOfWake';
import * as anchor from '@coral-xyz/anchor';
import { PublicKey } from '@solana/web3.js';

export default function Dashboard() {
  const { program, getChallengeAddress, getTreasuryAddress, account } = useProofOfWake();
  const [loading, setLoading] = useState(false);
  const [isWaking, setIsWaking] = useState(false);
  const [challengeData, setChallengeData] = useState<any>(null);

  const fetchChallenge = useCallback(async () => {
    if (!account) {
      setChallengeData(null);
      return;
    }

    try {
      setLoading(true);
      const challengeAddress = getChallengeAddress(new PublicKey(account.address));
      const data = await program.account.challenge.fetch(challengeAddress);
      setChallengeData(data);
    } catch (e) {
      console.log("No challenge found or error:", e.message);
      setChallengeData(null);
    } finally {
      setLoading(false);
    }
  }, [account, program]);

  useEffect(() => {
    fetchChallenge();
  }, [fetchChallenge]);

  const handleWakeUp = () => {
    if (!challengeData) {
      Alert.alert("Error", "No active challenge found. Start one first!");
      return;
    }
    setIsWaking(true);
  };

  const handleMathSolved = async () => {
    setIsWaking(false);
    try {
      setLoading(true);
      const userKey = new PublicKey(account!.address);
      const challengeAddress = getChallengeAddress(userKey);
      const treasuryAddress = getTreasuryAddress();

      const tx = await program.methods.completeDay().accounts({
        challenge: challengeAddress,
        treasury: treasuryAddress,
        authority: userKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      }).rpc();

      Alert.alert("Success", "Stake Secured! Streak continued. 🔥");
      await fetchChallenge();
    } catch (e) {
      console.error(e);
      Alert.alert("Failure", e.message);
    } finally {
      setLoading(false);
    }
  };

  const handleStartChallenge = async () => {
    try {
      setLoading(true);
      const userKey = new PublicKey(account!.address);
      const challengeAddress = getChallengeAddress(userKey);
      
      // Default to 7:00 AM, UTC (offset 0), 0.1 SOL stake
      const alarmHour = 7;
      const alarmMinute = 0;
      const timezoneOffset = 0;
      const stakeAmount = new anchor.BN(100_000_000); // 0.1 SOL

      await program.methods.startChallenge(
        alarmHour, 
        alarmMinute, 
        timezoneOffset, 
        stakeAmount
      ).accounts({
        challenge: challengeAddress,
        authority: userKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      }).rpc();

      Alert.alert("Challenge Started!", "Good luck with your 21-day journey.");
      await fetchChallenge();
    } catch (e) {
      console.error(e);
      Alert.alert("Start Error", e.message);
    } finally {
      setLoading(false);
    }
  };

  if (isWaking) {
    return <MathLock onSuccess={handleMathSolved} />;
  }

  return (
    <View className="flex-1 bg-white dark:bg-black">
      <Header />
      <ScrollView className="flex-1 p-6">
        <Text className="text-zinc-500 dark:text-zinc-400 text-sm font-bold uppercase tracking-widest mb-6">Active Challenge</Text>
        
        {loading && !challengeData ? (
          <ActivityIndicator size="large" color="#9945FF" />
        ) : challengeData && challengeData.isActive ? (
          <>
            <ChallengeCard 
              day={challengeData.streak + 1} 
              totalDays={21} 
              streak={challengeData.streak} 
              stake={challengeData.stakeAmount.toNumber() / 1e9} 
              reward={6.9} 
            />

            <View className="mt-8">
              <Text className="text-zinc-500 dark:text-zinc-400 text-sm font-bold uppercase tracking-widest mb-4">Actions</Text>
              
              <Pressable 
                onPress={handleWakeUp}
                disabled={loading}
                className="bg-solana-purple p-6 rounded-3xl items-center shadow-xl active:scale-[0.98]"
                style={{ backgroundColor: '#9945FF', opacity: loading ? 0.5 : 1 }}
              >
                <Text className="text-white text-xl font-black">{loading ? "PROCESSING..." : "WAKE UP 🔔"}</Text>
                <Text className="text-white/60 text-sm mt-1">Prove discipline, secure stake</Text>
              </Pressable>

              <View className="mt-4 p-6 bg-zinc-100 dark:bg-zinc-900 rounded-3xl border border-zinc-200 dark:border-zinc-800">
                <Text className="text-zinc-900 dark:text-white font-bold mb-2">Next Window</Text>
                <Text className="text-zinc-500 dark:text-zinc-400">Alarm set for {challengeData.alarmHour.toString().padStart(2, '0')}:{challengeData.alarmMinute.toString().padStart(2, '0')}.</Text>
                <Text className="text-zinc-500 dark:text-zinc-400">You must solve the math within ±1 hour of the set time.</Text>
              </View>
            </>
          ) : (
            <View className="items-center py-10">
              <Text className="text-zinc-400 mb-6 text-center">No active challenge. Commit 0.1 SOL to start your journey.</Text>
              <Pressable 
                onPress={handleStartChallenge}
                disabled={loading || !account}
                className="bg-solana-green p-6 rounded-3xl items-center shadow-xl w-full"
                style={{ backgroundColor: '#14F195', opacity: (loading || !account) ? 0.5 : 1 }}
              >
                <Text className="text-black text-xl font-black">START CHALLENGE 🚀</Text>
                {!account && <Text className="text-black/60 text-xs mt-1">Connect wallet first</Text>}
              </Pressable>
            </View>
          )}
        </View>
      </ScrollView>
    </View>
  );
}
