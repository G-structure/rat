import Image from 'next/image'
import Link from 'next/link'
import TerminalCommand from './TerminalCommand'

export default function Hero() {
  return (
    <section className="relative min-h-screen flex items-center justify-center overflow-hidden">
      {/* Background gradient overlay */}
      <div className="absolute inset-0 bg-gradient-to-b from-omnara-dark via-gray-900/50 to-omnara-dark" />
      
      {/* Animated background elements */}
      <div className="absolute inset-0 overflow-hidden">
        <div className="absolute top-20 left-10 w-72 h-72 bg-omnara-purple/10 rounded-full blur-3xl animate-float" />
        <div className="absolute bottom-20 right-10 w-96 h-96 bg-omnara-blue/10 rounded-full blur-3xl animate-float animation-delay-2000" />
      </div>
      
      <div className="relative z-10 max-w-6xl mx-auto px-4 py-12 md:py-20 text-center">
        {/* Logo with cheese decorations */}
        <div className="relative inline-block">
          <h1 className="text-5xl sm:text-6xl md:text-8xl font-bold mb-4 md:mb-6 tracking-wider">
            RAT MOBILE
          </h1>
          {/* Floating cheese emojis */}
          <span className="absolute -top-4 -left-8 md:-left-12 text-3xl md:text-5xl animate-float">🧀</span>
          <span className="absolute -top-2 -right-8 md:-right-12 text-2xl md:text-4xl animate-float animation-delay-2000">🧀</span>
          <span className="absolute -bottom-2 left-1/4 text-xl md:text-3xl animate-float animation-delay-4000">🧀</span>
        </div>
        
        {/* Tagline */}
        <h2 className="text-lg sm:text-xl md:text-2xl text-gray-300 mb-4 md:mb-8 font-light px-4">
          Launch & Control Claude Code from Anywhere
        </h2>
        
        {/* RAT Link */}
        <p className="text-sm sm:text-base text-gray-400 mb-2">
          <a href="#" className="text-omnara-blue hover:text-omnara-yellow transition-colors underline">
            RAT: Remote Access Terminals
          </a>
        </p>
        
        {/* Subtitle */}
        <p className="text-sm sm:text-base text-gray-400 mb-8 md:mb-12 max-w-2xl mx-auto px-4">
          Store, edit and review your agent-built code from your phone.
        </p>
        
        {/* CTA Buttons */}
        <div className="flex flex-col sm:flex-row gap-4 justify-center items-center mb-16">
          <Link 
            href="#"
            className="bg-omnara-yellow text-black px-8 py-4 rounded-lg font-semibold hover:bg-yellow-400 transition-colors flex items-center gap-2"
          >
            <svg className="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
              <path d="M17.05 20.28c-.98.95-2.05.8-3.08.35-1.09-.46-2.09-.48-3.24 0-1.44.62-2.2.44-3.06-.35C2.79 15.25 3.51 7.59 9.05 7.31c1.35.07 2.29.74 3.08.8 1.18-.24 2.31-.93 3.57-.84 1.51.12 2.65.72 3.4 1.8-3.12 1.87-2.38 5.98.48 7.13-.57 1.5-1.31 2.99-2.54 4.09l.01-.01zM12.03 7.25c-.15-2.23 1.66-4.07 3.74-4.25.29 2.58-2.34 4.5-3.74 4.25z"/>
            </svg>
            Download for iOS
          </Link>
          <Link 
            href="#"
            className="bg-white/10 backdrop-blur text-white px-8 py-4 rounded-lg font-semibold hover:bg-white/20 transition-colors flex items-center gap-2 border border-white/20"
          >
            <svg className="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
              <path d="M17.523 15.340s-.059-.003-.084-.009l-5.439-1.558v3.886l2.647 1.328c.234.117.498.046.644-.132.070-.085.104-.193.092-.306l-.002-3.204c0-.003.002-.006.002-.006zM7.477 8.660s.059.003.084.009l5.439 1.558V6.341L10.353 5.013c-.234-.117-.498-.046-.644.132-.070.085-.104.193-.092.306l.002 3.204c0 .003-.002.006-.002.006zM21.961 11.361l-2.213-.634c-.234-.067-.488.025-.603.220-.058.099-.078.214-.053.326l.892 3.913c.025.112.094.209.186.268s.203.077.308.049l2.213-.634c.234-.067.399-.281.399-.518V11.880c0-.237-.165-.451-.399-.518zM2.039 12.639l2.213.634c.234.067.488-.025.603-.220.058-.099.078-.214.053-.326l-.892-3.913c-.025-.112-.094-.209-.186-.268s-.203-.077-.308-.049l-2.213.634c-.234.067-.399.281-.399.518v2.471c0 .237.165.451.399.518z"/>
            </svg>
            Get it on Google Play
          </Link>
        </div>
        
        {/* Install prompt */}
        <div className="mt-8">
          <TerminalCommand />
        </div>
      </div>
      
      {/* Scroll indicator */}
      <div className="absolute bottom-8 left-1/2 transform -translate-x-1/2 animate-bounce">
        <svg className="w-6 h-6 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
        </svg>
      </div>
    </section>
  )
}