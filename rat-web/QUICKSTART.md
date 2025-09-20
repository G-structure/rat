# Quick Start Guide

## Fix Installation Issues

1. **Make sure you're in the correct directory:**
   ```bash
   cd /Users/aqeelali/Documents/vol2/rat/rat-web
   ```

2. **Clean install dependencies:**
   ```bash
   # Remove old dependencies
   rm -rf node_modules pnpm-lock.yaml package-lock.json yarn.lock
   
   # Install fresh
   pnpm install
   ```

3. **Start development server:**
   ```bash
   # Simple dev server (no Cloudflare features)
   pnpm dev
   ```

## Common Issues

### "wrangler: command not found"
This is expected for `pnpm dev` - it runs without wrangler. Use `pnpm dev:cf` only if you need Cloudflare features.

### Package not found errors
The `@codemirror/lang-typescript` package doesn't exist separately - TypeScript support is included in `@codemirror/lang-javascript`.

### Working directory issues
Make sure you're running commands from `/Users/aqeelali/Documents/vol2/rat/rat-web`, not from the Trash folder.

## Next Steps

1. The app will run on http://localhost:5173
2. You'll see the landing page with "Sign in with GitHub"
3. For full functionality, set up a GitHub OAuth app and add credentials to `.env`