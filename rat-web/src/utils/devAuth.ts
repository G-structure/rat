import { set } from "idb-keyval";
import type { User } from "../stores/authStore";

// Development authentication bypass
export async function setupDevAuth() {
  if (import.meta.env.DEV) {
    const devUser: User = {
      id: "dev-user-123",
      email: "dev@example.com",
      name: "Development User",
      avatar: "https://avatars.githubusercontent.com/u/12345678",
      githubConnected: true,
      claudeConnected: true,
    };
    
    await set("auth-user", devUser);
    await set("claude-token", "dev-claude-token-123");
    
    console.log("🧀 Dev auth setup complete! Refresh to authenticate.");
  }
}

// Call this in console to bypass auth in development:
// setupDevAuth()
