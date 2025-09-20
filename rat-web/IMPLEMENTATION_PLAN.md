# SolidStart Mobile IDE – Implementation Plan

> Transform rat-web from a WebSocket-based ACP client into a mobile-first Progressive Web App with GitHub integration, AI-powered code editing, and Cloudflare deployment.

---

## 🎯 Project Goals

1. **Mobile-First IDE**: Build a touch-optimized code editor that works seamlessly on phones
2. **GitHub Integration**: Use Device Code authentication for secure, mobile-friendly login
3. **AI-Powered Editing**: Integrate with AI agents for code generation and refactoring
4. **Progressive Web App**: Full offline support with installability
5. **Cloudflare Deployment**: Leverage edge computing for global performance

---

## 📋 Current State Analysis

### Existing Assets (rat-web)
- **Components**: ChatView, PlanPanel, TerminalView, CommandsPanel, ModeSelector, PermissionDialog, DiffView
- **State Management**: Simple signals with localStorage persistence
- **WebSocket Client**: ACP protocol implementation
- **UI Framework**: SolidJS with Vite

### Migration Strategy
- Preserve component logic where possible
- Adapt WebSocket patterns to SSE/REST
- Enhance mobile UX while maintaining functionality
- Incremental migration to avoid breaking changes

---

## 🏗️ Architecture Overview

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Mobile PWA    │────▶│ Cloudflare Edge  │────▶│  GitHub API     │
│  (SolidStart)   │     │   (Workers/KV)   │     │                 │
└─────────────────┘     └──────────────────┘     └─────────────────┘
         │                       │                         │
         ▼                       ▼                         ▼
   ┌──────────┐           ┌──────────┐            ┌──────────┐
   │IndexedDB │           │ Sessions │            │   Repos  │
   │  Cache   │           │    KV    │            │   Files  │
   └──────────┘           └──────────┘            └──────────┘
```

---

## 📁 Target Directory Structure

```
rat-web/
├── app/                        # SolidStart app directory
│   ├── routes/
│   │   ├── index.tsx          # Landing page with PWA install
│   │   ├── login.tsx          # GitHub Device Code flow
│   │   ├── dashboard.tsx      # Main IDE interface
│   │   ├── repos/
│   │   │   └── [...slug].tsx  # Dynamic repo/file routes
│   │   └── api/
│   │       ├── auth/
│   │       │   ├── device.start.ts
│   │       │   ├── device.poll.ts
│   │       │   └── logout.ts
│   │       ├── github/
│   │       │   ├── repos.ts
│   │       │   ├── files.[...path].ts
│   │       │   └── commits.ts
│   │       ├── agent/
│   │       │   ├── diff.ts
│   │       │   └── suggest.ts
│   │       └── sse/
│   │           └── runs.[id].ts
│   ├── components/
│   │   ├── Editor/
│   │   │   ├── CodeMirror.tsx
│   │   │   ├── DiffGutter.tsx
│   │   │   └── MobileToolbar.tsx
│   │   ├── Repo/
│   │   │   ├── FileTree.tsx
│   │   │   ├── BranchSelector.tsx
│   │   │   └── CommitList.tsx
│   │   ├── Mobile/
│   │   │   ├── BottomSheet.tsx
│   │   │   ├── SwipeableViews.tsx
│   │   │   └── SafeArea.tsx
│   │   ├── Agent/
│   │   │   ├── PromptInput.tsx
│   │   │   ├── RunsDisplay.tsx
│   │   │   └── DiffPreview.tsx
│   │   └── Common/
│   │       ├── Toast.tsx
│   │       ├── Loading.tsx
│   │       └── ErrorBoundary.tsx
│   ├── lib/
│   │   ├── api/
│   │   │   ├── github.ts
│   │   │   └── agent.ts
│   │   ├── auth/
│   │   │   ├── session.ts
│   │   │   └── device-code.ts
│   │   ├── store/
│   │   │   ├── queries.ts
│   │   │   ├── mutations.ts
│   │   │   └── ui-store.ts
│   │   ├── utils/
│   │   │   ├── diff-parser.ts
│   │   │   ├── selector-engine.ts
│   │   │   └── idb-cache.ts
│   │   └── contracts/
│   │       ├── surgical-edit.ts
│   │       └── dom-ops.ts
│   ├── styles/
│   │   ├── globals.css
│   │   ├── themes/
│   │   │   ├── dark.css
│   │   │   └── light.css
│   │   └── components/
│   └── manifest.webmanifest
├── public/
│   ├── icons/
│   │   ├── pwa-192.png
│   │   ├── pwa-512.png
│   │   └── maskable-1024.png
│   └── offline.html
├── worker-configuration.d.ts
├── wrangler.toml
├── vite.config.ts
├── tailwind.config.ts
├── postcss.config.cjs
├── tsconfig.json
├── .env.example
└── package.json
```

---

## 📦 Dependencies Update

### Core Framework
```json
{
  "dependencies": {
    "@solidjs/meta": "^0.29.0",
    "@solidjs/router": "^0.15.0",
    "solid-js": "^1.8.17",
    "solid-start": "^0.3.10",
    "solid-start-cloudflare-pages": "^0.3.10"
  }
}
```

### UI & Styling
```json
{
  "dependencies": {
    "tailwindcss": "^3.4.10",
    "tailwindcss-animate": "^1.0.7",
    "tailwind-merge": "^2.5.2",
    "@radix-ui/colors": "^3.0.0"
  }
}
```

### State Management
```json
{
  "dependencies": {
    "@tanstack/solid-query": "^5.56.0",
    "zod": "^3.23.8"
  }
}
```

### Editor & Code
```json
{
  "dependencies": {
    "codemirror": "^6.0.1",
    "@codemirror/lang-javascript": "^6.2.1",
    "@codemirror/lang-typescript": "^6.4.1",
    "@codemirror/lang-python": "^6.1.6",
    "@codemirror/lang-rust": "^6.0.1",
    "@codemirror/view": "^6.35.0",
    "@codemirror/state": "^6.4.0",
    "@codemirror/merge": "^6.7.0",
    "diff": "^7.0.0"
  }
}
```

### Storage & PWA
```json
{
  "dependencies": {
    "idb-keyval": "^6.2.1",
    "workbox-window": "^7.1.0"
  },
  "devDependencies": {
    "vite-plugin-pwa": "^0.20.5"
  }
}
```

---

## 🔧 Configuration Files

### wrangler.toml
```toml
name = "rat-mobile-ide"
compatibility_date = "2025-09-01"
pages_build_output_dir = ".solid/cloudflare"

[[kv_namespaces]]
binding = "SESSIONS"
id = "YOUR_KV_NAMESPACE_ID"

[[kv_namespaces]]
binding = "CACHE"
id = "YOUR_CACHE_NAMESPACE_ID"

[vars]
GITHUB_CLIENT_ID = "${GITHUB_CLIENT_ID}"
GITHUB_CLIENT_SECRET = "${GITHUB_CLIENT_SECRET}"
APP_BASE_URL = "https://ide.yourdomain.com"
AI_AGENT_URL = "${AI_AGENT_URL}"

[placement]
mode = "smart"
```

### vite.config.ts
```typescript
import { defineConfig } from 'vite';
import solid from 'solid-start/vite';
import cloudflare from 'solid-start-cloudflare-pages';
import { VitePWA } from 'vite-plugin-pwa';

export default defineConfig({
  plugins: [
    solid({ adapter: cloudflare() }),
    VitePWA({
      registerType: 'autoUpdate',
      includeAssets: ['icons/*', 'offline.html'],
      manifest: {
        name: 'RAT Mobile IDE',
        short_name: 'RAT IDE',
        description: 'AI-powered mobile code editor',
        start_url: '/',
        display: 'standalone',
        orientation: 'portrait',
        background_color: '#0b0b0b',
        theme_color: '#0b0b0b',
        icons: [
          {
            src: 'icons/pwa-192.png',
            sizes: '192x192',
            type: 'image/png'
          },
          {
            src: 'icons/pwa-512.png',
            sizes: '512x512',
            type: 'image/png'
          },
          {
            src: 'icons/maskable-1024.png',
            sizes: '1024x1024',
            type: 'image/png',
            purpose: 'maskable'
          }
        ]
      },
      workbox: {
        globPatterns: ['**/*.{js,css,html,ico,png,svg,woff2}'],
        runtimeCaching: [
          {
            urlPattern: /^https:\/\/api\.github\.com\/.*/i,
            handler: 'NetworkFirst',
            options: {
              cacheName: 'github-api-cache',
              expiration: {
                maxEntries: 50,
                maxAgeSeconds: 60 * 5 // 5 minutes
              }
            }
          }
        ]
      }
    })
  ]
});
```

---

## 🔑 Authentication Flow

### GitHub Device Code Implementation

1. **Start Device Code Flow**
   ```typescript
   // app/routes/api/auth/device.start.ts
   export const onRequestPost = async (event) => {
     const { GITHUB_CLIENT_ID } = event.env;
     const response = await fetch('https://github.com/login/device/code', {
       method: 'POST',
       headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
       body: new URLSearchParams({
         client_id: GITHUB_CLIENT_ID,
         scope: 'repo read:user'
       })
     });
     
     const data = await response.json();
     // Store device_code in KV with TTL
     await event.env.SESSIONS.put(
       `device:${data.device_code}`,
       JSON.stringify({ created: Date.now() }),
       { expirationTtl: data.expires_in }
     );
     
     return new Response(JSON.stringify(data));
   };
   ```

2. **Poll for Authorization**
   ```typescript
   // app/routes/api/auth/device.poll.ts
   export const onRequestPost = async (event) => {
     const { device_code } = await event.request.json();
     // Check GitHub for authorization
     // Create session on success
     // Return session cookie
   };
   ```

---

## 🎨 Mobile UI Components

### Bottom Sheet for Prompts
```typescript
// app/components/Mobile/BottomSheet.tsx
export function PromptSheet() {
  const [height, setHeight] = createSignal(300);
  const [isDragging, setIsDragging] = createSignal(false);
  
  return (
    <div 
      class="fixed inset-x-0 bottom-0 bg-black/90 backdrop-blur-xl rounded-t-3xl safe"
      style={{ height: `${height()}px` }}
    >
      <div class="drag-handle" onTouchStart={handleDragStart}>
        <div class="w-12 h-1 bg-white/20 rounded-full mx-auto my-3" />
      </div>
      <div class="p-4 space-y-4">
        <textarea
          class="w-full p-3 bg-white/10 rounded-xl resize-none"
          placeholder="Describe what you want to change..."
          rows={3}
        />
        <div class="flex gap-2">
          <button class="flex-1 py-3 bg-white/20 rounded-xl">
            🎤 Voice
          </button>
          <button class="flex-1 py-3 bg-blue-600 rounded-xl">
            ⚡ Generate
          </button>
        </div>
      </div>
    </div>
  );
}
```

### Swipeable Code Diff View
```typescript
// app/components/Agent/DiffPreview.tsx
export function DiffPreview(props: { diff: string }) {
  const [showSideBySide, setShowSideBySide] = createSignal(false);
  
  return (
    <div class="diff-container" onSwipeLeft={acceptDiff} onSwipeRight={rejectDiff}>
      <Show when={!showSideBySide()}>
        <UnifiedDiffView diff={props.diff} />
      </Show>
      <Show when={showSideBySide()}>
        <SideBySideDiffView diff={props.diff} />
      </Show>
    </div>
  );
}
```

---

## 🤖 Agent Integration

### Surgical Edit Contracts

#### Unified Diff Format
```json
{
  "format": "unified-diff@1",
  "files": [
    {
      "path": "src/components/Button.tsx",
      "patch": "--- a/src/components/Button.tsx\n+++ b/src/components/Button.tsx\n@@ -5,7 +5,7 @@\n export function Button({ children, onClick }) {\n   return (\n-    <button class=\"btn\" onClick={onClick}>\n+    <button class=\"btn btn-primary\" onClick={onClick}>\n       {children}\n     </button>\n   )"
    }
  ]
}
```

#### DOM Operations Format
```json
{
  "format": "dom-ops@1",
  "ops": [
    {
      "op": "setText",
      "selector": "#status-message",
      "text": "Changes saved"
    },
    {
      "op": "addClass",
      "selector": ".diff-preview",
      "class": "highlight-changes"
    },
    {
      "op": "insertAfter",
      "selector": "#prompt-input",
      "html": "<div class='hint'>Press ⌘K for commands</div>"
    }
  ]
}
```

### Agent Endpoint Implementation
```typescript
// app/routes/api/agent/diff.ts
export const onRequestPost = async (event) => {
  const { prompt, context } = await event.request.json();
  
  // Call AI agent with prompt and file context
  const response = await fetch(event.env.AI_AGENT_URL, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${event.env.AI_AGENT_KEY}`
    },
    body: JSON.stringify({
      messages: [
        { role: 'system', content: SURGICAL_EDIT_PROMPT },
        { role: 'user', content: formatUserPrompt(prompt, context) }
      ]
    })
  });
  
  const result = await response.json();
  return new Response(JSON.stringify(parseSurgicalEdit(result)));
};
```

---

## 📱 PWA Features

### Service Worker Strategy
- **Offline Shell**: Cache critical assets for offline access
- **Background Sync**: Queue prompts when offline
- **Update Notifications**: Prompt users for app updates

### Mobile Optimizations
- **Touch Gestures**: Swipe to navigate, pinch to zoom
- **Safe Areas**: Handle device notches and home indicators
- **Adaptive Icons**: Platform-specific app icons
- **Orientation Lock**: Portrait mode for better mobile UX

---

## 🚀 Implementation Phases

### Phase 1: Foundation (Days 1-3)
- [x] Create IMPLEMENTATION_PLAN.md
- [ ] Set up SolidStart project structure
- [ ] Configure Cloudflare and Tailwind
- [ ] Create basic routing structure

### Phase 2: Authentication (Days 4-5)
- [ ] Implement GitHub Device Code flow
- [ ] Create login UI with device code display
- [ ] Set up session management in KV

### Phase 3: Core UI (Days 6-8)
- [ ] Build mobile-optimized layout
- [ ] Implement bottom sheet for prompts
- [ ] Create file browser interface
- [ ] Add CodeMirror editor

### Phase 4: Agent Integration (Days 9-11)
- [ ] Create agent API endpoints
- [ ] Implement diff parser
- [ ] Build DOM operation executor
- [ ] Add SSE streaming for runs

### Phase 5: PWA & Polish (Days 12-14)
- [ ] Configure PWA manifest
- [ ] Implement service worker
- [ ] Add offline support
- [ ] Mobile gesture controls

### Phase 6: Testing & Deployment (Days 15-16)
- [ ] Unit tests for critical paths
- [ ] Mobile device testing
- [ ] Performance optimization
- [ ] Deploy to Cloudflare Pages

---

## 🔒 Security Considerations

### Authentication
- Device Code flow prevents credential exposure
- Session tokens stored in httpOnly cookies
- Automatic token refresh before expiry

### Content Security
- Strict CSP headers
- DOM operation allowlist
- Input sanitization for all user content
- XSS prevention in diff rendering

### Rate Limiting
- Per-session API limits
- Cloudflare rate limiting rules
- Exponential backoff for GitHub API

---

## 📊 Monitoring & Analytics

### Error Tracking
- Structured error taxonomy
- Client-side error boundary
- Server-side error logging

### Performance Metrics
- Core Web Vitals tracking
- API response times
- Agent processing duration

### Usage Analytics
- Anonymous usage statistics
- Feature adoption tracking
- Error rate monitoring

---

## 🔄 Migration from Current rat-web

### Preserve
- Component logic and structure
- State management patterns
- User interaction flows

### Replace
- WebSocket → SSE + REST APIs
- Local auth → GitHub OAuth
- Desktop UI → Mobile-first design

### Enhance
- Add offline support
- Implement code intelligence
- Mobile gestures and animations

---

## 📝 Development Commands

```bash
# Install dependencies
pnpm install

# Local development with Cloudflare
pnpm dev

# Build for production
pnpm build

# Deploy to Cloudflare Pages
pnpm deploy

# Run tests
pnpm test

# Type checking
pnpm typecheck
```

---

## 🎯 Success Criteria

1. **Mobile Performance**: < 2s time to interactive on 4G
2. **Offline Capability**: Core features work without connection
3. **User Experience**: 4.5+ app store rating
4. **Code Quality**: 90%+ test coverage for critical paths
5. **Adoption**: 1000+ active users within 3 months

---

## 🚦 Next Steps

1. Create `.env.example` with required environment variables
2. Set up GitHub OAuth app for Device Code flow
3. Create Cloudflare KV namespaces
4. Begin Phase 1 implementation

---

This plan provides a comprehensive roadmap for transforming rat-web into a production-ready mobile IDE. Each phase builds upon the previous, ensuring a stable migration path while adding powerful new features.