# RAT Mobile IDE

AI-powered mobile code editor with GitHub integration, built with SolidStart and deployed on Cloudflare Pages.

## Prerequisites

- Node.js 18+ or Bun
- pnpm, npm, or bun package manager
- Cloudflare account (for deployment)
- GitHub OAuth App (for authentication)

## Installation

```bash
# Install dependencies
pnpm install
# or
npm install
# or
bun install
```

## Setup

1. **Create a GitHub OAuth App**
   - Go to https://github.com/settings/applications/new
   - Set Authorization callback URL to: `http://localhost:5173/auth/callback`
   - Note your Client ID and Client Secret

2. **Configure Environment Variables**
   ```bash
   cp .env.example .env
   ```
   Edit `.env` and add your credentials:
   - `GITHUB_CLIENT_ID`
   - `GITHUB_CLIENT_SECRET`
   - `AI_AGENT_URL` (optional for AI features)
   - `AI_AGENT_KEY` (optional for AI features)

3. **Create Cloudflare KV Namespaces** (for deployment)
   ```bash
   npx wrangler kv:namespace create SESSIONS
   npx wrangler kv:namespace create CACHE
   ```
   Add the IDs to your `wrangler.toml`

## Development

```bash
# Start dev server (without Cloudflare features)
pnpm dev
# or with Cloudflare Workers emulation
pnpm dev:cf
```

The app will be available at http://localhost:5173

## Building

```bash
# Build for production
pnpm build:cf

# Preview production build
pnpm preview:cf
```

## Deployment

```bash
# Deploy to Cloudflare Pages
pnpm deploy
```

## Features

- 📱 Mobile-first design with PWA support
- 🔐 Secure GitHub Device Code authentication
- 📝 CodeMirror editor with syntax highlighting
- 🤖 AI-powered code suggestions and edits
- 📁 GitHub repository browsing
- ⚡ Edge computing with Cloudflare Workers
- 🌐 Offline support with service workers

## Project Structure

```
app/
├── components/     # Reusable UI components
├── routes/         # SolidStart file-based routing
│   ├── api/       # API endpoints
│   └── ...        # Page routes
├── styles/        # Global styles
└── lib/           # Utilities and helpers
```

## Troubleshooting

If you get "wrangler: command not found":
```bash
# Install wrangler globally
npm install -g wrangler
# or use npx
npx wrangler pages dev . -- vite
```

## License

MIT