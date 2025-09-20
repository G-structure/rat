import Image from 'next/image'

export default function Features() {
  return (
    <section className="relative py-24 px-4 overflow-hidden">
      {/* Background gradient */}
      <div className="absolute inset-0 bg-gradient-to-b from-omnara-dark via-blue-900/10 to-omnara-dark" />
      
      <div className="relative z-10 max-w-6xl mx-auto">
        <div className="relative inline-block w-full text-center">
          <h2 className="text-3xl sm:text-4xl md:text-6xl font-bold text-center mb-12 md:mb-16 uppercase tracking-wider px-4">
            AI IN YOUR POCKET
          </h2>
          {/* Cheese decorations */}
          <span className="absolute top-0 left-4 md:left-20 text-2xl md:text-4xl opacity-50">🧀</span>
          <span className="absolute top-0 right-4 md:right-20 text-2xl md:text-4xl opacity-50">🧀</span>
        </div>
        
        {/* Phone mockups */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8 md:gap-12 px-4 md:px-0">
          {/* Launch Agent */}
          <div className="text-center">
            <div className="relative mx-auto w-56 sm:w-64 h-[450px] sm:h-[500px] mb-6">
              <div className="absolute inset-0 bg-gradient-to-br from-blue-900/20 to-purple-900/20 rounded-[3rem] border border-white/10 backdrop-blur">
                <div className="p-6 pt-12">
                  <div className="bg-white/5 rounded-2xl p-4 mb-4">
                    <div className="text-omnara-blue text-sm mb-2">Start prompting</div>
                    <div className="bg-white/10 rounded h-2 w-3/4 mb-2"></div>
                    <div className="bg-white/10 rounded h-2 w-1/2"></div>
                  </div>
                  <div className="bg-omnara-yellow/20 rounded-2xl p-4 mb-4 border border-omnara-yellow/30">
                    <div className="text-omnara-yellow text-xs mb-2">Claude is typing...</div>
                    <div className="flex gap-1">
                      <div className="w-2 h-2 bg-omnara-yellow rounded-full animate-pulse"></div>
                      <div className="w-2 h-2 bg-omnara-yellow rounded-full animate-pulse animation-delay-200"></div>
                      <div className="w-2 h-2 bg-omnara-yellow rounded-full animate-pulse animation-delay-400"></div>
                    </div>
                  </div>
                  <div className="text-gray-400 text-xs">Loading preview...</div>
                </div>
                <div className="absolute bottom-6 left-1/2 -translate-x-1/2">
                  <button className="bg-omnara-yellow text-black px-6 py-2 rounded-full text-sm font-semibold">
                    Send
                  </button>
                </div>
              </div>
            </div>
            <h3 className="text-lg sm:text-xl font-semibold mb-2">Launch Agent</h3>
            <p className="text-gray-400 text-xs sm:text-sm">Control Claude from your phone</p>
          </div>
          
          {/* Track & Monitor */}
          <div className="text-center">
            <div className="relative mx-auto w-56 sm:w-64 h-[450px] sm:h-[500px] mb-6">
              <div className="absolute inset-0 bg-gradient-to-br from-blue-900/20 to-purple-900/20 rounded-[3rem] border border-white/10 backdrop-blur">
                <div className="p-6 pt-12">
                  <div className="text-omnara-blue text-sm mb-4">In Progress</div>
                  <div className="space-y-3">
                    <div className="bg-white/5 rounded-xl p-3 border border-green-500/30">
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-xs text-green-400">✓ Setup project</span>
                        <span className="text-xs text-gray-500">2m ago</span>
                      </div>
                      <div className="bg-green-500/20 rounded h-1"></div>
                    </div>
                    <div className="bg-white/5 rounded-xl p-3 border border-blue-500/30">
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-xs text-blue-400">◐ Building components</span>
                        <span className="text-xs text-gray-500">now</span>
                      </div>
                      <div className="bg-blue-500/20 rounded h-1 w-2/3"></div>
                    </div>
                    <div className="bg-white/5 rounded-xl p-3">
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-xs text-gray-400">○ Deploy to production</span>
                        <span className="text-xs text-gray-500">pending</span>
                      </div>
                      <div className="bg-white/10 rounded h-1 w-0"></div>
                    </div>
                  </div>
                  <div className="mt-6 bg-omnara-purple/20 rounded-xl p-3 border border-omnara-purple/30">
                    <div className="text-xs text-omnara-purple mb-1">Live Updates</div>
                    <div className="text-xs text-gray-400">Creating HomePage.tsx...</div>
                  </div>
                </div>
              </div>
            </div>
            <h3 className="text-lg sm:text-xl font-semibold mb-2">Track & Monitor</h3>
            <p className="text-gray-400 text-xs sm:text-sm">Real-time build progress</p>
          </div>
          
          {/* Interact in Real-Time */}
          <div className="text-center">
            <div className="relative mx-auto w-56 sm:w-64 h-[450px] sm:h-[500px] mb-6">
              <div className="absolute inset-0 bg-gradient-to-br from-blue-900/20 to-purple-900/20 rounded-[3rem] border border-white/10 backdrop-blur">
                <div className="p-6 pt-12">
                  <div className="text-omnara-blue text-sm mb-4">Code Review</div>
                  <div className="bg-white/5 rounded-xl p-3 mb-4">
                    <pre className="text-xs text-left overflow-hidden">
                      <code className="text-blue-400">{'function'}</code>{' '}
                      <code className="text-green-400">createUser</code>
                      <code className="text-gray-400">{'() {'}</code>
                      {'\n  '}
                      <code className="text-purple-400">const</code>{' '}
                      <code className="text-gray-300">user</code>{' = '}
                      <code className="text-gray-400">{'{'}</code>
                      {'\n    '}
                      <code className="text-gray-300">id:</code>{' '}
                      <code className="text-yellow-400">uuid()</code>
                      {'\n  '}
                      <code className="text-gray-400">{'}'}</code>
                    </pre>
                  </div>
                  <div className="flex gap-2 mb-4">
                    <button className="bg-green-500/20 text-green-400 px-3 py-1 rounded text-xs">
                      Approve
                    </button>
                    <button className="bg-yellow-500/20 text-yellow-400 px-3 py-1 rounded text-xs">
                      Request Changes
                    </button>
                  </div>
                  <div className="bg-white/5 rounded-xl p-3">
                    <input 
                      type="text" 
                      placeholder="Add a comment..."
                      className="bg-transparent text-sm w-full outline-none placeholder:text-gray-500"
                    />
                  </div>
                </div>
              </div>
            </div>
            <h3 className="text-lg sm:text-xl font-semibold mb-2">Interact in Real-Time</h3>
            <p className="text-gray-400 text-xs sm:text-sm">Review and iterate on the go</p>
          </div>
        </div>
      </div>
    </section>
  )
}