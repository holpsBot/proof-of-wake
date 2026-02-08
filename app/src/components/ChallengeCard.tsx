import React from 'react';
import { View, Text } from 'react-native';

interface ChallengeCardProps {
  day: number;
  totalDays: number;
  streak: number;
  stake: number;
  reward: number;
}

export function ChallengeCard({ day, totalDays, streak, stake, reward }: ChallengeCardProps) {
  const progress = (day / totalDays) * 100;

  return (
    <View className="bg-zinc-100 dark:bg-zinc-900 p-6 rounded-3xl border border-zinc-200 dark:border-zinc-800">
      <View className="flex-row justify-between items-end mb-6">
        <View>
          <Text className="text-zinc-500 dark:text-zinc-400 text-xs font-semibold uppercase tracking-widest mb-1">Current Progress</Text>
          <Text className="text-4xl font-extrabold text-zinc-900 dark:text-white">Day {day}</Text>
        </View>
        <Text className="text-zinc-500 dark:text-zinc-400 font-medium pb-1">of {totalDays}</Text>
      </View>

      {/* Progress Bar */}
      <View className="h-2 w-full bg-zinc-200 dark:bg-zinc-800 rounded-full overflow-hidden mb-8">
        <View 
          className="h-full bg-solana-green rounded-full" 
          style={{ width: `${progress}%`, backgroundColor: '#14F195' }} 
        />
      </View>

      <View className="flex-row justify-between">
        <View>
          <Text className="text-zinc-500 dark:text-zinc-400 text-xs mb-1">Locked Stake</Text>
          <Text className="text-lg font-bold text-zinc-900 dark:text-white">{stake} SOL</Text>
        </View>
        <View className="items-end">
          <Text className="text-zinc-500 dark:text-zinc-400 text-xs mb-1">Potential Reward</Text>
          <Text className="text-lg font-bold text-solana-green" style={{ color: '#14F195' }}>+{reward}%</Text>
        </View>
      </View>

      <View className="mt-6 pt-6 border-t border-zinc-200 dark:border-zinc-800 flex-row justify-between items-center">
        <Text className="text-zinc-500 dark:text-zinc-400 font-medium">Current Streak</Text>
        <View className="bg-orange-500/10 px-3 py-1 rounded-full">
          <Text className="text-orange-500 font-bold text-sm">🔥 {streak} Days</Text>
        </View>
      </View>
    </View>
  );
}
