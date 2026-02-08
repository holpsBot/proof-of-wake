import React, { useState } from 'react';
import { View, Text, ScrollView, Pressable } from 'react-native';
import { Header } from '../components/Header';
import { ChallengeCard } from '../components/ChallengeCard';
import { MathLock } from '../components/MathLock';

export default function Dashboard() {
  const [isWaking, setIsWaking] = useState(false);
  const [challenge, setChallenge] = useState({
    day: 8,
    totalDays: 21,
    streak: 8,
    stake: 0.1,
    reward: 6.9,
    isActive: true
  });

  const handleWakeUp = () => {
    setIsWaking(true);
  };

  const handleMathSolved = () => {
    setIsWaking(false);
    setChallenge(prev => ({
      ...prev,
      day: prev.day + 1,
      streak: prev.streak + 1
    }));
    alert('Stake Secured! Day ' + (challenge.day + 1) + ' complete. 🔥');
  };

  if (isWaking) {
    return <MathLock onSuccess={handleMathSolved} />;
  }

  return (
    <View className="flex-1 bg-white dark:bg-black">
      <Header />
      <ScrollView className="flex-1 p-6">
        <Text className="text-zinc-500 dark:text-zinc-400 text-sm font-bold uppercase tracking-widest mb-6">Active Challenge</Text>
        
        <ChallengeCard {...challenge} />

        <View className="mt-8">
          <Text className="text-zinc-500 dark:text-zinc-400 text-sm font-bold uppercase tracking-widest mb-4">Actions</Text>
          
          <Pressable 
            onPress={handleWakeUp}
            className="bg-solana-purple p-6 rounded-3xl items-center shadow-xl active:scale-[0.98]"
            style={{ backgroundColor: '#9945FF' }}
          >
            <Text className="text-white text-xl font-black">TEST ALARM 🔔</Text>
            <Text className="text-white/60 text-sm mt-1">Simulate morning cognitive lock</Text>
          </Pressable>

          <View className="mt-4 p-6 bg-zinc-100 dark:bg-zinc-900 rounded-3xl border border-zinc-200 dark:border-zinc-800">
            <Text className="text-zinc-900 dark:text-white font-bold mb-2">Next Window</Text>
            <Text className="text-zinc-500 dark:text-zinc-400">Alarm set for 07:00 AM.</Text>
            <Text className="text-zinc-500 dark:text-zinc-400">You must solve the math before 07:05 AM to avoid slashing.</Text>
          </View>
        </View>
      </ScrollView>
    </View>
  );
}
