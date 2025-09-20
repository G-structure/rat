import type { Metadata } from 'next'
import { Inter } from 'next/font/google'
import './globals.css'

const inter = Inter({ subsets: ['latin'] })

export const metadata: Metadata = {
  title: 'RAT MOBILE - Launch & Control Claude Code from Anywhere',
  description: 'Store, edit and review your agent-built codebase from your phone.',
  icons: {
    icon: '/favicon.ico',
  },
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className={`${inter.className} bg-omnara-dark text-white antialiased`}>
        {children}
      </body>
    </html>
  )
}