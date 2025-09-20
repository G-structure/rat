import Hero from '@/components/Landing/Hero'
import Features from '@/components/Landing/Features'
import AlwaysInTheLoop from '@/components/Landing/AlwaysInTheLoop'
import FooterCTA from '@/components/Landing/FooterCTA'

export default function Home() {
  return (
    <main className="min-h-screen bg-omnara-dark overflow-x-hidden">
      <Hero />
      <Features />
      <AlwaysInTheLoop />
      <FooterCTA />
    </main>
  )
}