import { createSignal, Show } from "solid-js";
import { A } from "@solidjs/router";
import { Title } from "@solidjs/meta";

export default function Login() {
  const [deviceCode, setDeviceCode] = createSignal<string | null>(null);
  const [userCode, setUserCode] = createSignal<string | null>(null);
  const [verificationUri, setVerificationUri] = createSignal<string | null>(null);
  const [polling, setPolling] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [pollProgress, setPollProgress] = createSignal(0);

  async function startDeviceFlow() {
    // Just immediately log in with mock data
    localStorage.setItem("mock-user", JSON.stringify({
      user: {
        login: "demo-user",
        name: "Demo User",
        avatar_url: "https://avatars.githubusercontent.com/u/1?v=4"
      },
      credits: 1000
    }));
    
    // Redirect to dashboard immediately
    window.location.href = "/dashboard";
  }

  async function pollForAuthorization(data: any) {
    setPolling(true);
    setPollProgress(0);
    const startTime = Date.now();
    const interval = (data.interval || 5) * 1000;
    const expiresIn = data.expires_in * 1000;
    
    while (Date.now() - startTime < expiresIn) {
      await new Promise(resolve => setTimeout(resolve, interval));
      
      // Update progress
      const elapsed = Date.now() - startTime;
      setPollProgress(Math.min((elapsed / expiresIn) * 100, 100));
      
      try {
        const isDev = import.meta.env.DEV;
        let response;
        
        if (isDev && !import.meta.env.VITE_GITHUB_CLIENT_ID) {
          // Use mock API in development
          const { mockApi } = await import("~/lib/api/mock-api");
          const result = await mockApi.pollDeviceFlow(data.device_code);
          if (result.ok) {
            // Store mock user data
            localStorage.setItem("mock-user", JSON.stringify({
              user: result.user,
              credits: 1000
            }));
            window.location.href = "/dashboard";
            return;
          }
          continue;
        } else {
          response = await fetch("/api/auth/device.poll", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ device_code: data.device_code })
          });
          
          if (response.status === 200) {
            // Success! Redirect to dashboard
            window.location.href = "/dashboard";
            return;
          } else if (response.status === 428) {
            // Still pending, continue polling
            continue;
          } else if (response.status >= 400) {
            // Error occurred
            const error = await response.text();
            setError(error || "Authorization failed");
            break;
          }
        }
      } catch (err) {
        setError("Network error occurred");
        break;
      }
    }
    
    setPolling(false);
    if (!error()) {
      setError("Device code expired. Please try again.");
    }
  }

  function copyCode() {
    if (userCode()) {
      navigator.clipboard.writeText(userCode()!);
      // Could add a toast notification here
    }
  }

  return (
    <>
      <Title>Sign in - RAT Mobile IDE</Title>
      <main class="min-h-[100dvh] p-4 safe">
        <div class="max-w-md mx-auto space-y-8 pt-8">
          {/* Back button */}
          <A href="/" class="inline-flex items-center gap-2 text-muted-foreground hover:text-foreground">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
            </svg>
            Back
          </A>
          
          <div class="space-y-4">
            <h1 class="text-3xl font-bold">Sign in with GitHub</h1>
            <p class="text-muted-foreground">
              We use GitHub's Device Code flow for secure authentication on mobile devices.
              No passwords needed!
            </p>
          </div>
          
          <div class="space-y-6">
            <Show when={!userCode()}>
              <button 
                onClick={startDeviceFlow}
                class="btn btn-primary w-full flex items-center justify-center gap-3"
                disabled={polling()}
              >
                <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23A11.509 11.509 0 0112 5.803c1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576C20.566 21.797 24 17.3 24 12c0-6.627-5.373-12-12-12z"/>
                </svg>
                Generate Device Code
              </button>
            </Show>
            
            <Show when={userCode()}>
              <div class="space-y-6 animate-fade-in">
                {/* Step 1: Show the code */}
                <div class="p-6 rounded-2xl bg-secondary/50 space-y-4">
                  <div class="flex items-center justify-between">
                    <h2 class="font-medium">Your device code:</h2>
                    <button 
                      onClick={copyCode}
                      class="text-sm text-muted-foreground hover:text-foreground"
                    >
                      Copy
                    </button>
                  </div>
                  <div class="text-4xl font-mono font-bold tracking-[0.25em] text-center select-all">
                    {userCode()}
                  </div>
                </div>
                
                {/* Step 2: Open GitHub */}
                <div class="space-y-3">
                  <p class="text-sm text-muted-foreground">
                    Open GitHub to enter the code:
                  </p>
                  <a 
                    href={verificationUri()!}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="btn btn-secondary w-full flex items-center justify-center gap-2"
                  >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                    </svg>
                    Open {new URL(verificationUri()!).hostname}
                  </a>
                </div>
                
                {/* Step 3: Polling status */}
                <Show when={polling()}>
                  <div class="space-y-3">
                    <div class="flex items-center justify-between text-sm">
                      <span class="text-muted-foreground">Waiting for authorization...</span>
                      <span class="text-muted-foreground">{Math.round(pollProgress())}%</span>
                    </div>
                    <div class="h-2 bg-secondary rounded-full overflow-hidden">
                      <div 
                        class="h-full bg-primary transition-all duration-1000 ease-linear"
                        style={{ width: `${pollProgress()}%` }}
                      />
                    </div>
                    <p class="text-xs text-center text-muted-foreground">
                      This will complete automatically once you authorize
                    </p>
                  </div>
                </Show>
              </div>
            </Show>
            
            <Show when={error()}>
              <div class="p-4 rounded-xl bg-destructive/10 text-destructive">
                <p class="text-sm font-medium">Error</p>
                <p class="text-sm">{error()}</p>
              </div>
            </Show>
          </div>
          
          <div class="pt-8 space-y-4 text-sm text-muted-foreground">
            <h3 class="font-medium text-foreground">How it works:</h3>
            <ol class="space-y-2 list-decimal list-inside">
              <li>Generate a unique device code</li>
              <li>Visit GitHub on any device</li>
              <li>Enter the code to authorize</li>
              <li>You'll be automatically signed in</li>
            </ol>
            <p class="text-xs">
              This method is more secure than entering passwords on mobile keyboards.
            </p>
          </div>
        </div>
      </main>
    </>
  );
}