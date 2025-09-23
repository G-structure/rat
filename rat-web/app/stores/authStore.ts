import { createSignal, createEffect } from "solid-js";
import { createStore } from "solid-js/store";
import { get, set } from "idb-keyval";

export interface User {
  id: string;
  email?: string;
  name?: string;
  avatar?: string;
  githubConnected: boolean;
  claudeConnected: boolean;
}

interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  githubDeviceCode?: string;
  githubVerificationUri?: string;
  githubUserCode?: string;
  claudeToken?: string;
}

const [authState, setAuthState] = createStore<AuthState>({
  user: null,
  isAuthenticated: false,
  isLoading: false,
});

// Initialize auth state from IndexedDB
async function initAuthState() {
  const savedUser = await get("auth-user");
  const savedToken = await get("claude-token");
  
  if (savedUser) {
    setAuthState({
      user: savedUser,
      isAuthenticated: true,
      claudeToken: savedToken,
    });
  }
}

// Call init on module load
initAuthState();

// GitHub Device Flow Authentication
export async function startGitHubAuth() {
  try {
    setAuthState("isLoading", true);
    
    const response = await fetch("/api/auth/device.start", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
    });
    
    if (!response.ok) throw new Error("Failed to start device flow");
    
    const data = await response.json();
    
    setAuthState({
      githubDeviceCode: data.device_code,
      githubVerificationUri: data.verification_uri,
      githubUserCode: data.user_code,
    });
    
    // Open GitHub authorization page
    window.open(data.verification_uri, "_blank");
    
    // Start polling
    pollGitHubAuth(data.device_code, data.interval || 5);
    
    return data;
  } catch (error) {
    console.error("GitHub auth error:", error);
    throw error;
  } finally {
    setAuthState("isLoading", false);
  }
}

async function pollGitHubAuth(deviceCode: string, interval: number) {
  const pollInterval = setInterval(async () => {
    try {
      const response = await fetch("/api/auth/device.poll", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ device_code: deviceCode }),
      });
      
      if (response.ok) {
        clearInterval(pollInterval);
        const userData = await response.json();
        
        const user: User = {
          id: userData.id,
          email: userData.email,
          name: userData.name,
          avatar: userData.avatar_url,
          githubConnected: true,
          claudeConnected: authState.user?.claudeConnected || false,
        };
        
        setAuthState({
          user,
          isAuthenticated: true,
          githubDeviceCode: undefined,
          githubVerificationUri: undefined,
          githubUserCode: undefined,
        });
        
        // Save to IndexedDB
        await set("auth-user", user);
        
        // Don't redirect - let onboarding flow handle navigation
      } else if (response.status === 401) {
        // Still pending, continue polling
      } else {
        // Error occurred
        clearInterval(pollInterval);
        throw new Error("Authentication failed");
      }
    } catch (error) {
      clearInterval(pollInterval);
      console.error("Polling error:", error);
    }
  }, interval * 1000);
}

// Claude Code Authentication
export async function startClaudeAuth() {
  try {
    setAuthState("isLoading", true);
    
    const response = await fetch("/api/auth/claude.start", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
    });
    
    if (!response.ok) throw new Error("Failed to start Claude device flow");
    
    const data = await response.json();
    
    // Open Claude authorization page
    window.open(data.verification_uri_complete, "_blank");
    
    // Start polling
    pollClaudeAuth(data.device_code, data.interval || 5);
    
    return data;
  } catch (error) {
    console.error("Claude auth error:", error);
    throw error;
  } finally {
    setAuthState("isLoading", false);
  }
}

async function pollClaudeAuth(deviceCode: string, interval: number) {
  const pollInterval = setInterval(async () => {
    try {
      const response = await fetch("/api/auth/claude.poll", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ device_code: deviceCode }),
      });
      
      if (response.ok) {
        clearInterval(pollInterval);
        const data = await response.json();
        
        setAuthState((state) => ({
          claudeToken: data.access_token,
          user: state.user ? {
            ...state.user,
            claudeConnected: true,
          } : null,
        }));
        
        // Save token
        await set("claude-token", data.access_token);
        
        // Save updated user
        if (authState.user) {
          await set("auth-user", authState.user);
        }
      } else if (response.status === 401) {
        // Still pending, continue polling
      } else {
        // Error occurred
        clearInterval(pollInterval);
        throw new Error("Claude authentication failed");
      }
    } catch (error) {
      clearInterval(pollInterval);
      console.error("Claude polling error:", error);
    }
  }, interval * 1000);
}

// Logout
export async function logout() {
  try {
    await fetch("/api/auth/logout", { method: "POST" });
    
    // Clear local state
    setAuthState({
      user: null,
      isAuthenticated: false,
      claudeToken: undefined,
    });
    
    // Clear IndexedDB
    await set("auth-user", null);
    await set("claude-token", null);
    
    window.location.href = "/login";
  } catch (error) {
    console.error("Logout error:", error);
  }
}

// Check authentication status
export async function checkAuth() {
  try {
    // First check local storage for saved user
    const savedUser = await get("auth-user");
    const savedToken = await get("claude-token");
    
    if (savedUser) {
      setAuthState({
        user: savedUser,
        isAuthenticated: true,
        claudeToken: savedToken,
      });
      
      // Try to verify with server
      try {
        const response = await fetch("/api/me");
        if (response.ok) {
          const userData = await response.json();
          const user: User = {
            id: userData.id,
            email: userData.email,
            name: userData.name,
            avatar: userData.avatar_url,
            githubConnected: true,
            claudeConnected: !!savedToken,
          };
          
          setAuthState({
            user,
            isAuthenticated: true,
            claudeToken: savedToken,
          });
          
          // Update saved user
          await set("auth-user", user);
          return true;
        }
      } catch (e) {
        // Server verification failed, but we have local data
        return true;
      }
    }
    
    // Try server auth check
    const response = await fetch("/api/me");
    
    if (response.ok) {
      const userData = await response.json();
      
      const user: User = {
        id: userData.id,
        email: userData.email,
        name: userData.name,
        avatar: userData.avatar_url,
        githubConnected: true,
        claudeConnected: !!savedToken,
      };
      
      setAuthState({
        user,
        isAuthenticated: true,
        claudeToken: savedToken,
      });
      
      // Save to local storage
      await set("auth-user", user);
      
      return true;
    }
    
    return false;
  } catch (error) {
    console.error("Auth check error:", error);
    
    // Check if we have local auth data as fallback
    const savedUser = await get("auth-user");
    if (savedUser) {
      setAuthState({
        user: savedUser,
        isAuthenticated: true,
        claudeToken: await get("claude-token"),
      });
      return true;
    }
    
    return false;
  }
}

export { authState };
