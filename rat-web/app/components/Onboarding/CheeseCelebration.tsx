import { Component, createSignal, onMount, For } from "solid-js";

interface Particle {
  id: number;
  x: number;
  y: number;
  rotation: number;
  scale: number;
  emoji: string;
  delay: number;
  duration: number;
}

interface CheeseCelebrationProps {
  onComplete?: () => void;
}

export const CheeseCelebration: Component<CheeseCelebrationProps> = (props) => {
  const [particles, setParticles] = createSignal<Particle[]>([]);
  const [showTapHint, setShowTapHint] = createSignal(false);
  
  const cheeseEmojis = ["🧀", "🧀", "🧀", "🧀", "🧀"];
  const celebrationEmojis = ["✨", "🎉", "⭐", "🌟", "💫", "🎊"];
  
  onMount(() => {
    // Generate confetti particles
    const newParticles: Particle[] = [];
    for (let i = 0; i < 60; i++) {
      const isCheeseEmoji = i < 40; // 40 cheese emojis, 20 celebration emojis
      newParticles.push({
        id: i,
        x: Math.random() * 100,
        y: Math.random() * 100,
        rotation: Math.random() * 360,
        scale: 0.5 + Math.random() * 0.5,
        emoji: isCheeseEmoji 
          ? cheeseEmojis[Math.floor(Math.random() * cheeseEmojis.length)]
          : celebrationEmojis[Math.floor(Math.random() * celebrationEmojis.length)],
        delay: Math.random() * 0.5,
        duration: 2 + Math.random() * 2,
      });
    }
    setParticles(newParticles);
    
    // Haptic feedback if available
    if ('vibrate' in navigator) {
      navigator.vibrate([200, 100, 200]);
    }
    
    // Show tap hint after animation
    setTimeout(() => setShowTapHint(true), 2500);
  });
  
  const handleTap = () => {
    props.onComplete?.();
  };
  
  return (
    <div 
      class="fixed inset-0 z-50 flex items-center justify-center bg-gradient-to-br from-yellow-400 via-orange-400 to-yellow-500 overflow-hidden cursor-pointer"
      onClick={handleTap}
      onTouchEnd={handleTap}
    >
      {/* Animated background rays */}
      <div class="absolute inset-0 overflow-hidden">
        <div class="rays-container">
          <For each={Array(12).fill(0)}>
            {(_, i) => (
              <div 
                class="ray"
                style={{
                  transform: `rotate(${i() * 30}deg)`,
                  "animation-delay": `${i() * 0.1}s`
                }}
              />
            )}
          </For>
        </div>
      </div>
      
      {/* Main "Say Cheese!" text */}
      <div class="cheese-text-container">
        <h1 class="cheese-text">
          <span class="cheese-word">Say</span>
          <span class="cheese-emoji-main">🧀</span>
          <span class="cheese-word">Cheese!</span>
        </h1>
        <div class="cheese-subtitle">
          <span class="subtitle-emoji">📸</span>
          <span class="subtitle-text">You're all set!</span>
          <span class="subtitle-emoji">✨</span>
        </div>
      </div>
      
      {/* Confetti particles */}
      <div class="absolute inset-0 pointer-events-none">
        <For each={particles()}>
          {(particle) => (
            <div
              class="confetti-particle"
              style={{
                left: `${particle.x}%`,
                top: `${particle.y}%`,
                transform: `rotate(${particle.rotation}deg) scale(${particle.scale})`,
                "animation-delay": `${particle.delay}s`,
                "animation-duration": `${particle.duration}s`
              }}
            >
              <span class="text-4xl">{particle.emoji}</span>
            </div>
          )}
        </For>
      </div>
      
      {/* Tap to continue hint */}
      <div class={`tap-hint ${showTapHint() ? "visible" : ""}`}>
        <p class="text-white/90 text-lg font-medium">Tap to continue</p>
      </div>
      
      <style>{`
        @keyframes rayRotate {
          from {
            transform: rotate(var(--rotation)) scale(0);
            opacity: 0;
          }
          50% {
            opacity: 0.3;
          }
          to {
            transform: rotate(var(--rotation)) scale(2);
            opacity: 0;
          }
        }
        
        @keyframes cheeseTextPop {
          0% {
            transform: scale(0) rotate(-10deg);
            opacity: 0;
          }
          50% {
            transform: scale(1.2) rotate(5deg);
          }
          70% {
            transform: scale(0.9) rotate(-2deg);
          }
          100% {
            transform: scale(1) rotate(0deg);
            opacity: 1;
          }
        }
        
        @keyframes cheeseBounce {
          0%, 100% {
            transform: translateY(0) rotate(0deg) scale(1);
          }
          25% {
            transform: translateY(-10px) rotate(-5deg) scale(1.1);
          }
          75% {
            transform: translateY(-10px) rotate(5deg) scale(1.1);
          }
        }
        
        @keyframes confettiFall {
          0% {
            transform: translateY(-100vh) rotate(0deg) scale(0);
            opacity: 0;
          }
          10% {
            opacity: 1;
          }
          100% {
            transform: translateY(100vh) rotate(720deg) scale(var(--scale));
            opacity: 0;
          }
        }
        
        @keyframes fadeInUp {
          from {
            opacity: 0;
            transform: translateY(20px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }
        
        .rays-container {
          position: absolute;
          inset: 0;
          display: flex;
          align-items: center;
          justify-content: center;
        }
        
        .ray {
          position: absolute;
          width: 200%;
          height: 100px;
          background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.3), transparent);
          animation: rayRotate 3s ease-out forwards;
          --rotation: 0deg;
        }
        
        .cheese-text-container {
          position: relative;
          z-index: 10;
          animation: cheeseTextPop 0.8s cubic-bezier(0.68, -0.55, 0.265, 1.55) forwards;
        }
        
        .cheese-text {
          font-size: 4rem;
          font-weight: 800;
          color: white;
          text-shadow: 0 4px 20px rgba(0, 0, 0, 0.2);
          display: flex;
          align-items: center;
          gap: 1rem;
          flex-wrap: wrap;
          justify-content: center;
          text-align: center;
          line-height: 1.2;
        }
        
        @media (min-width: 768px) {
          .cheese-text {
            font-size: 6rem;
          }
        }
        
        .cheese-word {
          display: inline-block;
        }
        
        .cheese-emoji-main {
          display: inline-block;
          font-size: 5rem;
          animation: cheeseBounce 2s ease-in-out infinite;
          animation-delay: 0.8s;
        }
        
        @media (min-width: 768px) {
          .cheese-emoji-main {
            font-size: 7rem;
          }
        }
        
        .cheese-subtitle {
          margin-top: 1.5rem;
          font-size: 1.5rem;
          color: rgba(255, 255, 255, 0.9);
          display: flex;
          align-items: center;
          gap: 1rem;
          animation: fadeInUp 0.8s ease forwards;
          animation-delay: 1s;
          opacity: 0;
        }
        
        .subtitle-emoji {
          font-size: 2rem;
          animation: pulse 2s ease-in-out infinite;
        }
        
        .subtitle-text {
          font-weight: 600;
        }
        
        @keyframes pulse {
          0%, 100% {
            transform: scale(1);
          }
          50% {
            transform: scale(1.2);
          }
        }
        
        .confetti-particle {
          position: absolute;
          animation: confettiFall var(--duration, 3s) ease-in forwards;
          animation-delay: var(--delay, 0s);
          --scale: var(--particle-scale, 1);
        }
        
        .tap-hint {
          position: absolute;
          bottom: 60px;
          left: 50%;
          transform: translateX(-50%);
          opacity: 0;
          transition: opacity 0.5s ease;
          animation: fadeInUp 0.5s ease forwards;
          animation-delay: 0.5s;
        }
        
        .tap-hint.visible {
          opacity: 1;
        }
        
        .tap-hint p {
          background: rgba(0, 0, 0, 0.3);
          padding: 12px 24px;
          border-radius: 30px;
          backdrop-filter: blur(10px);
        }
      `}</style>
    </div>
  );
};