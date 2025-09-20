export default function CityBackground() {
  return (
    <svg
      className="absolute inset-0 w-full h-full"
      preserveAspectRatio="xMidYMid slice"
      viewBox="0 0 1920 1080"
    >
      <defs>
        <linearGradient id="skyGradient" x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" stopColor="#0A0A0A" />
          <stop offset="100%" stopColor="#1a1a2e" />
        </linearGradient>
        
        <filter id="glow">
          <feGaussianBlur stdDeviation="4" result="coloredBlur"/>
          <feMerge>
            <feMergeNode in="coloredBlur"/>
            <feMergeNode in="SourceGraphic"/>
          </feMerge>
        </filter>
      </defs>
      
      <rect width="1920" height="1080" fill="url(#skyGradient)" />
      
      {/* Building silhouettes */}
      <g opacity="0.8">
        <rect x="100" y="600" width="120" height="480" fill="#0f0f23" />
        <rect x="250" y="500" width="100" height="580" fill="#0f0f23" />
        <rect x="380" y="550" width="140" height="530" fill="#0f0f23" />
        <rect x="550" y="400" width="80" height="680" fill="#0f0f23" />
        <rect x="660" y="450" width="160" height="630" fill="#0f0f23" />
        <rect x="850" y="520" width="110" height="560" fill="#0f0f23" />
        <rect x="990" y="380" width="130" height="700" fill="#0f0f23" />
        <rect x="1150" y="480" width="90" height="600" fill="#0f0f23" />
        <rect x="1270" y="420" width="150" height="660" fill="#0f0f23" />
        <rect x="1450" y="500" width="100" height="580" fill="#0f0f23" />
        <rect x="1580" y="450" width="120" height="630" fill="#0f0f23" />
        <rect x="1730" y="530" width="140" height="550" fill="#0f0f23" />
      </g>
      
      {/* Window lights */}
      <g filter="url(#glow)">
        {/* Building 1 */}
        <rect x="110" y="620" width="20" height="15" fill="#FCD34D" opacity="0.8" />
        <rect x="140" y="620" width="20" height="15" fill="#FCD34D" opacity="0.6" />
        <rect x="170" y="620" width="20" height="15" fill="#FCD34D" opacity="0.9" />
        <rect x="110" y="650" width="20" height="15" fill="#8B5CF6" opacity="0.7" />
        <rect x="140" y="680" width="20" height="15" fill="#FCD34D" opacity="0.8" />
        
        {/* Building 2 */}
        <rect x="560" y="420" width="15" height="15" fill="#3B82F6" opacity="0.9" />
        <rect x="590" y="420" width="15" height="15" fill="#3B82F6" opacity="0.6" />
        <rect x="560" y="450" width="15" height="15" fill="#FCD34D" opacity="0.8" />
        <rect x="590" y="480" width="15" height="15" fill="#8B5CF6" opacity="0.7" />
        
        {/* Building 3 */}
        <rect x="1000" y="400" width="25" height="20" fill="#FCD34D" opacity="0.9" />
        <rect x="1040" y="400" width="25" height="20" fill="#3B82F6" opacity="0.7" />
        <rect x="1080" y="400" width="25" height="20" fill="#FCD34D" opacity="0.8" />
        <rect x="1000" y="440" width="25" height="20" fill="#8B5CF6" opacity="0.6" />
        <rect x="1040" y="480" width="25" height="20" fill="#FCD34D" opacity="0.9" />
        
        {/* Scattered lights */}
        <circle cx="280" cy="540" r="3" fill="#FCD34D" opacity="0.8" />
        <circle cx="400" cy="580" r="3" fill="#3B82F6" opacity="0.9" />
        <circle cx="700" cy="490" r="3" fill="#8B5CF6" opacity="0.7" />
        <circle cx="880" cy="560" r="3" fill="#FCD34D" opacity="0.8" />
        <circle cx="1180" cy="510" r="3" fill="#3B82F6" opacity="0.9" />
        <circle cx="1300" cy="460" r="3" fill="#8B5CF6" opacity="0.8" />
        <circle cx="1480" cy="540" r="3" fill="#FCD34D" opacity="0.7" />
        <circle cx="1610" cy="490" r="3" fill="#3B82F6" opacity="0.8" />
        <circle cx="1760" cy="570" r="3" fill="#8B5CF6" opacity="0.9" />
      </g>
      
      {/* Animated lights */}
      <g>
        <circle cx="150" cy="700" r="2" fill="#FCD34D" opacity="0">
          <animate attributeName="opacity" values="0;1;0" dur="3s" repeatCount="indefinite" />
        </circle>
        <circle cx="600" cy="500" r="2" fill="#3B82F6" opacity="0">
          <animate attributeName="opacity" values="0;1;0" dur="2.5s" repeatCount="indefinite" begin="0.5s" />
        </circle>
        <circle cx="1050" cy="450" r="2" fill="#8B5CF6" opacity="0">
          <animate attributeName="opacity" values="0;1;0" dur="3.5s" repeatCount="indefinite" begin="1s" />
        </circle>
      </g>
    </svg>
  )
}