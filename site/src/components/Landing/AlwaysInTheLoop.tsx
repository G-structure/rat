export default function AlwaysInTheLoop() {
  return (
    <section className="relative py-24 px-4 overflow-hidden">
      {/* Background with floating elements */}
      <div className="absolute inset-0 bg-gradient-to-b from-omnara-dark via-purple-900/10 to-omnara-dark" />
      
      {/* Floating orbs */}
      <div className="absolute inset-0 overflow-hidden">
        <div className="absolute top-1/4 left-1/4 w-32 h-32 bg-omnara-purple/20 rounded-full blur-2xl animate-float" />
        <div className="absolute top-3/4 right-1/4 w-24 h-24 bg-omnara-blue/20 rounded-full blur-2xl animate-float animation-delay-2000" />
        <div className="absolute top-1/2 left-1/2 w-40 h-40 bg-omnara-yellow/10 rounded-full blur-3xl animate-float animation-delay-4000" />
      </div>
      
      <div className="relative z-10 max-w-6xl mx-auto">
        <h2 className="text-3xl sm:text-4xl md:text-6xl font-bold text-center mb-8 uppercase tracking-wider px-4">
          ALWAYS BE PROMPTING
        </h2>
        
        <div className="max-w-3xl mx-auto text-center px-4">
          <h3 className="text-xl sm:text-2xl md:text-3xl font-semibold mb-6 text-gradient">
            Never in the Dark.
          </h3>
          
          <p className="text-base sm:text-lg text-gray-400 mb-8">
            Get notified the moment agents start their magic.
          </p>
          
          <div className="text-gray-500 text-sm space-y-2 mb-8">
            <p>No surprises, buddy. In real time.</p>
            <p>Full audit, no silent cousins or missed steps.</p>
          </div>
          
          {/* New reset message */}
          <div className="bg-omnara-purple/10 backdrop-blur rounded-2xl p-6 border border-omnara-purple/30 mb-8 max-w-md mx-auto relative">
            <div className="flex items-center justify-center mb-3">
              <svg className="w-8 h-8 text-omnara-purple" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <p className="text-white font-semibold mb-2">Every 5 hours it resets.</p>
            <p className="text-gray-400 text-sm">Why would you ever not always be prompting?</p>
            {/* Sneaky cheese */}
            <span className="absolute -top-3 -right-3 text-2xl animate-pulse">🧀</span>
          </div>
          
          {/* Visual representation of notifications */}
          <div className="mt-16 grid grid-cols-1 md:grid-cols-3 gap-6 max-w-4xl mx-auto">
            <div className="bg-white/5 backdrop-blur rounded-2xl p-6 border border-white/10 hover:border-omnara-purple/50 transition-colors">
              <div className="w-12 h-12 bg-omnara-purple/20 rounded-xl flex items-center justify-center mb-4 mx-auto">
                <svg className="w-6 h-6 text-omnara-purple" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                </svg>
              </div>
              <h4 className="font-semibold mb-2">Instant Alerts</h4>
              <p className="text-sm text-gray-400">Get notified when agents start working on your code</p>
            </div>
            
            <div className="bg-white/5 backdrop-blur rounded-2xl p-6 border border-white/10 hover:border-omnara-blue/50 transition-colors">
              <div className="w-12 h-12 bg-omnara-blue/20 rounded-xl flex items-center justify-center mb-4 mx-auto">
                <svg className="w-6 h-6 text-omnara-blue" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </div>
              <h4 className="font-semibold mb-2">Full Transparency</h4>
              <p className="text-sm text-gray-400">Track every change and decision made by AI</p>
            </div>
            
            <div className="bg-white/5 backdrop-blur rounded-2xl p-6 border border-white/10 hover:border-omnara-yellow/50 transition-colors">
              <div className="w-12 h-12 bg-omnara-yellow/20 rounded-xl flex items-center justify-center mb-4 mx-auto">
                <svg className="w-6 h-6 text-omnara-yellow" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
              </div>
              <h4 className="font-semibold mb-2">Real-time Updates</h4>
              <p className="text-sm text-gray-400">Monitor progress as it happens, no delays</p>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}