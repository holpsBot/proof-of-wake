import React from 'react';
import { View, Text, Pressable } from 'react-native';
import { useMobileWallet } from '@wallet-ui/react-native-kit';

export function Header() {
  const { account, connect, disconnect } = useMobileWallet();

  return (
    <View className="flex-row justify-between items-center px-6 py-4 bg-white dark:bg-zinc-900 border-b border-zinc-200 dark:border-zinc-800">
      <View>
        <Text className="text-lg font-bold text-zinc-900 dark:text-white">Proof of Wake</Text>
        <Text className="text-xs text-zinc-500 dark:text-zinc-400">Habit Protocol</Text>
      </View>
      
      {account ? (
        <Pressable 
          onPress={disconnect}
          className="bg-zinc-100 dark:bg-zinc-800 px-3 py-1.5 rounded-full border border-zinc-200 dark:border-zinc-700"
        >
          <Text className="text-xs font-medium text-zinc-600 dark:text-zinc-300">
            {account.address.toString().slice(0, 4)}...{account.address.toString().slice(-4)}
          </Text>
        </Pressable>
      ) : (
        <Pressable 
          onPress={connect}
          className="bg-solana-purple px-4 py-2 rounded-full"
          style={{ backgroundColor: '#9945FF' }}
        >
          <Text className="text-xs font-bold text-white uppercase tracking-wider">Connect</Text>
        </Pressable>
      )}
    </View>
  );
}
