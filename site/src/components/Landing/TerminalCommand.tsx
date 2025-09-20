'use client'

import { useState } from 'react'

export default function TerminalCommand() {
  const [showMessage, setShowMessage] = useState(false)
  const command = "rat ins- 🖕 NAH WE GOT YOU FAM"
  const commandLine2 = "JUST AUTH AND PROMPT"

  const handleCopy = () => {
    navigator.clipboard.writeText(`${command} ${commandLine2}`)
    setShowMessage(true)
    setTimeout(() => setShowMessage(false), 3000)
  }

  return (
    <div className="relative inline-block max-w-full">
      <div className="bg-black/80 backdrop-blur rounded-lg border border-gray-700 overflow-hidden">
        <div className="flex items-center">
          {/* Terminal prompt */}
          <div className="flex-1 px-4 py-3">
            <div className="flex items-start">
              <span className="text-green-400 text-xs sm:text-sm font-mono mr-2">$</span>
              <div className="text-center sm:text-left flex-1">
                <code className="text-gray-300 text-xs sm:text-sm font-mono block">
                  {command}
                </code>
                <code className="text-gray-300 text-xs sm:text-sm font-mono block">
                  {commandLine2}
                </code>
              </div>
            </div>
          </div>
          
          {/* Copy button */}
          <button
            onClick={handleCopy}
            className={`px-3 sm:px-4 py-3 transition-all duration-300 self-stretch border-l border-gray-700 ${
              showMessage ? 'bg-omnara-purple text-white' : 'bg-gray-800 hover:bg-gray-700 text-gray-400'
            }`}
          >
            {showMessage ? (
              <span className="text-xs sm:text-sm font-bold whitespace-nowrap">WE DON'T GIVE A RAT'S ASS</span>
            ) : (
              <svg className="w-4 h-4 sm:w-5 sm:h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
            )}
          </button>
        </div>
      </div>
      
      {/* Terminal window decoration */}
      <div className="absolute -top-6 left-0 flex items-center gap-2 px-3">
        <div className="w-3 h-3 rounded-full bg-red-500"></div>
        <div className="w-3 h-3 rounded-full bg-yellow-500"></div>
        <div className="w-3 h-3 rounded-full bg-green-500"></div>
      </div>
    </div>
  )
}