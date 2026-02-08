import React, { useState, useEffect } from 'react';
import { View, Text, Pressable } from 'react-native';

interface MathLockProps {
  onSuccess: () => void;
}

export function MathLock({ onSuccess }: MathLockProps) {
  const [problem, setProblem] = useState({ q: '', a: 0 });
  const [input, setInput] = useState('');

  useEffect(() => {
    generateProblem();
  }, []);

  const generateProblem = () => {
    const num1 = Math.floor(Math.random() * 20) + 1;
    const num2 = Math.floor(Math.random() * 10) + 1;
    const num3 = Math.floor(Math.random() * 15) + 1;
    setProblem({
      q: `(${num1} × ${num2}) + ${num3}`,
      a: (num1 * num2) + num3
    });
  };

  const handlePress = (num: string) => {
    setInput(prev => prev + num);
  };

  const handleClear = () => setInput('');

  const handleSubmit = () => {
    if (parseInt(input) === problem.a) {
      onSuccess();
    } else {
      alert('Wrong answer! Brain still foggy? Try again.');
      setInput('');
      generateProblem();
    }
  };

  return (
    <View className="flex-1 bg-black items-center justify-center p-8">
      <Text className="text-solana-green text-sm font-bold uppercase tracking-[4px] mb-4" style={{ color: '#14F195' }}>Cognitive Lock Active</Text>
      <Text className="text-white text-6xl font-black mb-12">{problem.q}</Text>
      
      <View className="w-full mb-12">
        <View className="h-20 bg-zinc-900 border-2 border-zinc-800 rounded-2xl items-center justify-center">
          <Text className="text-white text-4xl font-mono">{input || '_'}</Text>
        </View>
      </View>

      <View className="flex-row flex-wrap justify-center gap-4 max-w-[300px]">
        {['1', '2', '3', '4', '5', '6', '7', '8', '9', 'C', '0', 'OK'].map((btn) => (
          <Pressable
            key={btn}
            onPress={() => {
              if (btn === 'C') handleClear();
              else if (btn === 'OK') handleSubmit();
              else handlePress(btn);
            }}
            className={`w-16 h-16 rounded-full items-center justify-center ${btn === 'OK' ? 'bg-solana-green' : 'bg-zinc-800'}`}
            style={btn === 'OK' ? { backgroundColor: '#14F195' } : {}}
          >
            <Text className={`text-xl font-bold ${btn === 'OK' ? 'text-black' : 'text-white'}`}>{btn}</Text>
          </Pressable>
        ))}
      </View>
    </View>
  );
}
