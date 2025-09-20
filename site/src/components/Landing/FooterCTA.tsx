import Link from 'next/link'
import CityBackground from './CityBackground'
import TerminalCommand from './TerminalCommand'

export default function FooterCTA() {
  return (
    <section className="relative py-24 px-4 overflow-hidden">
      {/* Background image placeholder */}
      <div className="absolute inset-0 bg-gradient-to-t from-omnara-dark via-omnara-dark/80 to-transparent" />
      <div className="absolute inset-0 opacity-20">
        <CityBackground />
      </div>
      
      <div className="relative z-10 max-w-6xl mx-auto text-center px-4">
        <h2 className="text-3xl sm:text-4xl md:text-6xl font-bold mb-8 uppercase tracking-wider">
          READY TO TAKE<br />COMMAND?
        </h2>
        
        <p className="text-lg sm:text-xl text-gray-300 mb-8 md:mb-12 max-w-2xl mx-auto">
          Join the revolution of mobile-first AI development. Your code, your rules, anywhere you go.
        </p>
        
        {/* CTA Buttons */}
        <div className="flex flex-col sm:flex-row gap-4 justify-center items-center mb-16">
          <Link 
            href="#"
            className="bg-omnara-yellow text-black px-8 py-4 rounded-lg font-semibold hover:bg-yellow-400 transition-colors flex items-center gap-2 shadow-lg hover:shadow-omnara-yellow/20"
          >
            <svg className="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
              <path d="M17.05 20.28c-.98.95-2.05.8-3.08.35-1.09-.46-2.09-.48-3.24 0-1.44.62-2.2.44-3.06-.35C2.79 15.25 3.51 7.59 9.05 7.31c1.35.07 2.29.74 3.08.8 1.18-.24 2.31-.93 3.57-.84 1.51.12 2.65.72 3.4 1.8-3.12 1.87-2.38 5.98.48 7.13-.57 1.5-1.31 2.99-2.54 4.09l.01-.01zM12.03 7.25c-.15-2.23 1.66-4.07 3.74-4.25.29 2.58-2.34 4.5-3.74 4.25z"/>
            </svg>
            Download Now
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
        
        {/* Command line install */}
        <div className="mb-16">
          <p className="text-sm text-gray-500 mb-4">Or install via command line:</p>
          <TerminalCommand />
        </div>
        
        {/* Joke/Tagline */}
        <div className="text-gray-400 italic relative">
          <p className="text-lg mb-2">
            "Because the best code is written from the beach... 🏖️"
          </p>
          <p className="text-sm">
            (Results may vary. Beach not included. Please code responsibly. 🧀)
          </p>
          {/* Hidden cheese trail */}
          <span className="absolute -left-20 top-0 text-xl opacity-30">🧀</span>
          <span className="absolute -right-20 top-0 text-xl opacity-30">🧀</span>
        </div>
        
        {/* Footer links */}
        <div className="mt-16 pt-8 border-t border-white/10">
          <div className="flex flex-wrap justify-center gap-6 text-sm text-gray-400">
            <Link href="#" className="hover:text-white transition-colors">Privacy Policy</Link>
            <Link href="#" className="hover:text-white transition-colors">Terms of Service</Link>
            <Link href="#" className="hover:text-white transition-colors">Documentation</Link>
            <Link href="#" className="hover:text-white transition-colors">GitHub</Link>
            <Link href="#" className="hover:text-white transition-colors">Contact</Link>
          </div>
          <p className="mt-6 text-xs text-gray-500">
            © 2024 OMNARA. All rights reserved. Built with Claude Code.
          </p>
        </div>
      </div>
    </section>
  )
}