import { A } from "@solidjs/router";
import { Title } from "@solidjs/meta";

export default function Home() {
  return (
    <>
      <Title>RAT Mobile IDE - AI-Powered Code Editor</Title>
      <main class="min-h-[100dvh] grid place-items-center p-4 safe">
        <div class="max-w-md w-full space-y-8 text-center">
          {/* Logo/Icon placeholder */}
          <div class="mx-auto w-24 h-24 bg-gradient-to-br from-blue-500 to-purple-600 rounded-3xl flex items-center justify-center">
            <span class="text-4xl font-bold text-white">&lt;/&gt;</span>
          </div>
          
          <div class="space-y-4">
            <h1 class="text-4xl font-bold bg-gradient-to-r from-blue-400 to-purple-600 bg-clip-text text-transparent">
              RAT Mobile IDE
            </h1>
            <p class="text-lg text-muted-foreground">
              AI-powered code editing on your phone. 
              <br />
              <span class="text-sm">Prompt → Diff → Commit → Ship.</span>
            </p>
          </div>
          
          <div class="space-y-4 pt-8">
            <button 
              onClick={() => {
                // Just immediately log in with mock data
                localStorage.setItem("mock-user", JSON.stringify({
                  user: {
                    login: "demo-user",
                    name: "Demo User",
                    avatar_url: "https://avatars.githubusercontent.com/u/1?v=4"
                  },
                  credits: 1000
                }));
                window.location.href = "/dashboard";
              }}
              class="btn btn-primary w-full flex items-center justify-center gap-3"
            >
              <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23A11.509 11.509 0 0112 5.803c1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576C20.566 21.797 24 17.3 24 12c0-6.627-5.373-12-12-12z"/>
              </svg>
              Sign in with GitHub
            </button>
            
            <p class="text-sm text-muted-foreground">
              Uses secure Device Code authentication
              <br />
              <span class="text-xs opacity-70">No passwords on mobile</span>
            </p>
          </div>
          
          <div class="pt-12 space-y-2">
            <h2 class="text-sm font-medium text-muted-foreground">Features</h2>
            <div class="grid grid-cols-2 gap-3 text-sm">
              <div class="p-3 rounded-xl bg-secondary/50">
                <div class="text-2xl mb-1">🤖</div>
                <div>AI-Powered</div>
              </div>
              <div class="p-3 rounded-xl bg-secondary/50">
                <div class="text-2xl mb-1">📱</div>
                <div>Mobile-First</div>
              </div>
              <div class="p-3 rounded-xl bg-secondary/50">
                <div class="text-2xl mb-1">🔐</div>
                <div>Secure Auth</div>
              </div>
              <div class="p-3 rounded-xl bg-secondary/50">
                <div class="text-2xl mb-1">⚡</div>
                <div>PWA Ready</div>
              </div>
            </div>
          </div>
          
          {/* Install PWA prompt (shown conditionally) */}
          <div class="fixed bottom-4 inset-x-4 p-4 bg-primary text-primary-foreground rounded-2xl shadow-lg hidden" id="install-prompt">
            <div class="flex items-center justify-between">
              <div class="pr-3">
                <p class="font-medium">Install RAT IDE</p>
                <p class="text-sm opacity-90">Add to your home screen</p>
              </div>
              <button class="btn bg-white/20 hover:bg-white/30 text-white px-6">
                Install
              </button>
            </div>
          </div>
        </div>
      </main>
    </>
  );
}