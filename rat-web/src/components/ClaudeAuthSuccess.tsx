import { Component, createSignal, onMount, For } from "solid-js";

interface CheeseParticle {
  id: number;
  x: number;
  y: number;
  size: number;
  delay: number;
  duration: number;
}

interface ClaudeAuthSuccessProps {
  onComplete?: () => void;
}

export const ClaudeAuthSuccess: Component<ClaudeAuthSuccessProps> = (props) => {
  const [particles, setParticles] = createSignal<CheeseParticle[]>([]);
  const [isAnimating, setIsAnimating] = createSignal(true);
  
  onMount(() => {
    // Generate cheese particles
    const newParticles: CheeseParticle[] = [];
    for (let i = 0; i < 20; i++) {
      newParticles.push({
        id: i,
        x: 10 + Math.random() * 80,
        y: 10 + Math.random() * 80,
        size: 20 + Math.random() * 30,
        delay: Math.random() * 0.5,
        duration: 1 + Math.random() * 0.5,
      });
    }
    setParticles(newParticles);
    
    // Auto advance after animation
    setTimeout(() => {
      setIsAnimating(false);
      props.onComplete?.();
    }, 3000);
  });
  
  return (
    <div class="claude-auth-success">
      <div class="success-content">
        <div class="success-icon-container">
          <svg class="success-checkmark" viewBox="0 0 100 100">
            <circle cx="50" cy="50" r="45" fill="none" stroke="currentColor" stroke-width="4" class="checkmark-circle-success" />
            <path d="M30 50l15 15 30-30" fill="none" stroke="currentColor" stroke-width="4" stroke-linecap="round" stroke-linejoin="round" class="checkmark-path-success" />
          </svg>
          <div class="claude-logo">
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
            </svg>
          </div>
        </div>
        
        <h2 class="success-title">Claude Code Connected!</h2>
        <p class="success-message">CLI authentication successful</p>
        
        {/* Cheese particles */}
        <div class="cheese-particles">
          <For each={particles()}>
            {(particle) => (
              <span
                class="cheese-particle-small"
                style={{
                  left: `${particle.x}%`,
                  top: `${particle.y}%`,
                  "font-size": `${particle.size}px`,
                  "animation-delay": `${particle.delay}s`,
                  "animation-duration": `${particle.duration}s`
                }}
              >
                🧀
              </span>
            )}
          </For>
        </div>
      </div>
      
      <style>{`
        .claude-auth-success {
          position: fixed;
          inset: 0;
          z-index: 100;
          background: rgba(0, 0, 0, 0.8);
          backdrop-filter: blur(10px);
          display: flex;
          align-items: center;
          justify-content: center;
          animation: fadeIn 0.3s ease-out;
        }
        
        @keyframes fadeIn {
          from {
            opacity: 0;
          }
          to {
            opacity: 1;
          }
        }
        
        .success-content {
          text-align: center;
          padding: 3rem;
          position: relative;
          z-index: 10;
        }
        
        .success-icon-container {
          position: relative;
          width: 120px;
          height: 120px;
          margin: 0 auto 2rem;
        }
        
        .success-checkmark {
          width: 100%;
          height: 100%;
          color: #10b981;
        }
        
        .checkmark-circle-success {
          stroke-dasharray: 283;
          stroke-dashoffset: 283;
          animation: drawCircle 0.6s ease-out forwards;
        }
        
        .checkmark-path-success {
          stroke-dasharray: 100;
          stroke-dashoffset: 100;
          animation: drawCheck 0.4s ease-out 0.6s forwards;
        }
        
        @keyframes drawCircle {
          to {
            stroke-dashoffset: 0;
          }
        }
        
        @keyframes drawCheck {
          to {
            stroke-dashoffset: 0;
          }
        }
        
        .claude-logo {
          position: absolute;
          bottom: -10px;
          right: -10px;
          width: 40px;
          height: 40px;
          background: linear-gradient(135deg, #ff6b00, #ff8533);
          border-radius: 12px;
          padding: 8px;
          color: white;
          animation: popIn 0.5s cubic-bezier(0.68, -0.55, 0.265, 1.55) 0.8s backwards;
        }
        
        @keyframes popIn {
          from {
            transform: scale(0) rotate(-45deg);
          }
          to {
            transform: scale(1) rotate(0deg);
          }
        }
        
        .success-title {
          font-size: 2rem;
          font-weight: 700;
          color: white;
          margin-bottom: 0.5rem;
          animation: slideInUp 0.6s ease-out 0.4s backwards;
        }
        
        .success-message {
          font-size: 1.125rem;
          color: rgba(255, 255, 255, 0.8);
          animation: slideInUp 0.6s ease-out 0.5s backwards;
        }
        
        @keyframes slideInUp {
          from {
            opacity: 0;
            transform: translateY(20px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }
        
        .cheese-particles {
          position: absolute;
          inset: -50px;
          pointer-events: none;
          overflow: hidden;
        }
        
        .cheese-particle-small {
          position: absolute;
          animation: floatCheese var(--duration, 1.5s) ease-out var(--delay, 0s) backwards;
        }
        
        @keyframes floatCheese {
          0% {
            transform: translateY(0) rotate(0deg) scale(0);
            opacity: 0;
          }
          20% {
            opacity: 1;
          }
          100% {
            transform: translateY(-100px) rotate(360deg) scale(1);
            opacity: 0;
          }
        }
      `}</style>
    </div>
  );
};