# Running the Development Server

## Steps to Run

1. **Clean install dependencies:**
   ```bash
   cd /Users/aqeelali/Documents/vol2/rat/rat-web
   rm -rf node_modules pnpm-lock.yaml
   pnpm install
   ```

2. **Start the development server:**
   ```bash
   pnpm dev
   ```

3. **Open in browser:**
   - Go to http://localhost:5173
   - You should see the landing page with "Sign in with GitHub"

## What's Working

- ✅ Landing page with mobile-first design
- ✅ Mock authentication flow (click "Sign in with GitHub" to test)
- ✅ Dashboard with mock data
- ✅ Basic routing between pages
- ✅ Mobile-optimized UI components

## Development Mode

The app runs with mock data when no GitHub credentials are configured. This allows you to:
- Test the UI without backend setup
- See how the authentication flow works
- Navigate through the app structure

## Next Steps

To enable real GitHub integration:
1. Create a GitHub OAuth App
2. Copy `.env.example` to `.env`
3. Add your GitHub credentials
4. Restart the dev server