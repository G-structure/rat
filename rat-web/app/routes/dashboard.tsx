import { createSignal, Show, onMount, createEffect } from "solid-js";
import { createQuery } from "@tanstack/solid-query";
import { Title } from "@solidjs/meta";
import { chatStore } from "~/stores/chatStore";

export default function Dashboard() {
  const [selectedRepo, setSelectedRepo] = createSignal<string | null>(null);
  const [showPromptSheet, setShowPromptSheet] = createSignal(false);
  const [, setRefresh] = createSignal(0);
  
  // Query for user data
  const userQuery = createQuery(() => ({
    queryKey: ["me"],
    queryFn: async () => {
      const isDev = import.meta.env.DEV;
      
      if (isDev && !import.meta.env.VITE_GITHUB_CLIENT_ID) {
        // Use mock API in development
        const { mockApi } = await import("~/lib/api/mock-api");
        const data = await mockApi.getMe();
        if (!data.user) {
          window.location.href = "/login";
          throw new Error("Unauthorized");
        }
        return data;
      } else {
        const response = await fetch("/api/me");
        if (!response.ok) {
          if (response.status === 401) {
            window.location.href = "/login";
            throw new Error("Unauthorized");
          }
          throw new Error("Failed to fetch user data");
        }
        return response.json();
      }
    }
  }));
  
  // Enhanced mock repos with more details
  const repos = [
    { 
      name: "react-todo-app", 
      language: "TypeScript", 
      updated: "2h ago",
      description: "A modern todo app with React and TypeScript",
      stars: 234,
      issues: 5
    },
    { 
      name: "python-api-server", 
      language: "Python", 
      updated: "1d ago",
      description: "RESTful API server with FastAPI and PostgreSQL",
      stars: 89,
      issues: 12
    },
    { 
      name: "mobile-shopping-app", 
      language: "Swift", 
      updated: "3d ago",
      description: "iOS shopping app with SwiftUI",
      stars: 456,
      issues: 3
    },
    {
      name: "ml-recommendation-engine",
      language: "Python",
      updated: "5d ago", 
      description: "Machine learning recommendation system",
      stars: 1023,
      issues: 8
    },
    {
      name: "vue-dashboard",
      language: "JavaScript",
      updated: "1w ago",
      description: "Analytics dashboard built with Vue 3",
      stars: 567,
      issues: 15
    }
  ];
  
  // Refresh UI when chat states change
  createEffect(() => {
    const states = chatStore.projectStates();
    setRefresh(prev => prev + 1);
  });
  
  onMount(() => {
    // Add swipe gesture for prompt sheet
    let startY = 0;
    const handleTouchStart = (e: TouchEvent) => {
      startY = e.touches[0].clientY;
    };
    
    const handleTouchMove = (e: TouchEvent) => {
      const currentY = e.touches[0].clientY;
      if (startY - currentY > 50 && !showPromptSheet()) {
        setShowPromptSheet(true);
      }
    };
    
    document.addEventListener("touchstart", handleTouchStart);
    document.addEventListener("touchmove", handleTouchMove);
    
    return () => {
      document.removeEventListener("touchstart", handleTouchStart);
      document.removeEventListener("touchmove", handleTouchMove);
    };
  });
  
  return (
    <>
      <Title>Dashboard - RAT Mobile IDE</Title>
      <div class="h-[100dvh] flex flex-col bg-background">
        {/* Header */}
        <header class="flex items-center justify-between p-4 border-b border-border safe-top">
          <div class="flex items-center gap-3">
            <Show when={userQuery.data?.user?.avatar_url}>
              <img 
                src={userQuery.data.user.avatar_url}
                alt="Avatar"
                class="w-8 h-8 rounded-full"
              />
            </Show>
            <div>
              <h1 class="font-semibold">
                {userQuery.data?.user?.login || "Loading..."}
              </h1>
              <p class="text-xs text-muted-foreground">
                {userQuery.data?.credits || 0} credits
              </p>
            </div>
          </div>
          
          <button class="p-2 rounded-lg hover:bg-secondary">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </button>
        </header>
        
        {/* Main content area */}
        <div class="flex-1 overflow-hidden">
          <Show when={!selectedRepo()}>
            {/* Repository list */}
            <div class="h-full overflow-y-auto">
              <div class="p-4 space-y-4">
                <h2 class="text-lg font-semibold">Recent Repositories</h2>
                <div class="space-y-2">
                  {repos.map(repo => (
                    <button
                      onClick={() => {
                        setSelectedRepo(repo.name);
                        // Simulate loading and redirect to editor
                        setTimeout(() => {
                          window.location.href = `/app?repo=${repo.name}`;
                        }, 500);
                      }}
                      class="w-full p-4 rounded-xl bg-secondary/50 hover:bg-secondary transition-colors text-left"
                    >
                      <div class="flex items-center justify-between">
                        <div class="flex-1">
                          <div class="flex items-center gap-2">
                            <h3 class="font-medium">{repo.name}</h3>
                            {(() => {
                              const state = chatStore.getProjectState(repo.name);
                              return state !== "idle" && (
                                <div
                                  class={`w-2 h-2 rounded-full ${
                                    state === "generating" ? "bg-yellow-500 animate-pulse" :
                                    state === "completed" ? "bg-green-500" :
                                    "bg-gray-400 border border-gray-600"
                                  }`}
                                  title={
                                    state === "generating" ? "Generating..." :
                                    state === "completed" ? "Completed" :
                                    "Idle"
                                  }
                                />
                              );
                            })()}
                          </div>
                          <p class="text-xs text-muted-foreground mt-1">
                            {repo.description}
                          </p>
                          <div class="flex items-center gap-4 mt-2 text-xs text-muted-foreground">
                            <span class="flex items-center gap-1">
                              <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                                <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z"/>
                              </svg>
                              {repo.stars}
                            </span>
                            <span class="flex items-center gap-1">
                              <div class={`w-3 h-3 rounded-full ${
                                repo.language === "TypeScript" ? "bg-blue-500" :
                                repo.language === "Python" ? "bg-yellow-500" :
                                repo.language === "Swift" ? "bg-orange-500" :
                                repo.language === "JavaScript" ? "bg-yellow-400" :
                                "bg-gray-500"
                              }`} />
                              {repo.language}
                            </span>
                            <span>Updated {repo.updated}</span>
                          </div>
                        </div>
                        <svg class="w-5 h-5 text-muted-foreground flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                        </svg>
                      </div>
                    </button>
                  ))}
                </div>
                
                <button class="w-full p-4 rounded-xl border border-dashed border-border hover:border-muted-foreground transition-colors">
                  <div class="flex items-center justify-center gap-2 text-muted-foreground">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                    </svg>
                    <span>Browse all repositories</span>
                  </div>
                </button>
              </div>
              
              {/* Quick actions */}
              <div class="p-4 space-y-4 border-t border-border">
                <h2 class="text-lg font-semibold">Quick Actions</h2>
                <div class="grid grid-cols-2 gap-3">
                  <a href="/app" class="p-4 rounded-xl bg-primary text-primary-foreground text-center">
                    <div class="text-2xl mb-2">💻</div>
                    <div class="text-sm font-medium">Open Editor</div>
                  </a>
                  <button class="p-4 rounded-xl bg-secondary">
                    <div class="text-2xl mb-2">🔍</div>
                    <div class="text-sm font-medium">Search Code</div>
                  </button>
                  <button class="p-4 rounded-xl bg-secondary">
                    <div class="text-2xl mb-2">📝</div>
                    <div class="text-sm font-medium">Recent Edits</div>
                  </button>
                  <button class="p-4 rounded-xl bg-secondary">
                    <div class="text-2xl mb-2">🤖</div>
                    <div class="text-sm font-medium">AI History</div>
                  </button>
                </div>
              </div>
            </div>
          </Show>
          
          <Show when={selectedRepo()}>
            {/* Repository view (placeholder) */}
            <div class="h-full flex flex-col">
              <div class="p-4 border-b border-border">
                <button 
                  onClick={() => setSelectedRepo(null)}
                  class="flex items-center gap-2 text-muted-foreground hover:text-foreground"
                >
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
                  </svg>
                  <span>{selectedRepo()}</span>
                </button>
              </div>
              <div class="flex-1 p-4">
                <p class="text-muted-foreground">File browser will go here...</p>
              </div>
            </div>
          </Show>
        </div>
        
        {/* Floating action button */}
        <button
          onClick={() => setShowPromptSheet(true)}
          class="fixed bottom-6 right-6 w-14 h-14 rounded-full bg-primary text-primary-foreground shadow-lg flex items-center justify-center safe"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
        </button>
        
        {/* Prompt sheet (placeholder) */}
        <Show when={showPromptSheet()}>
          <div 
            class="fixed inset-0 bg-black/50 z-40"
            onClick={() => setShowPromptSheet(false)}
          />
          <div class="fixed inset-x-0 bottom-0 bg-background border-t border-border rounded-t-3xl p-6 safe animate-slide-up z-50">
            <div class="drag-handle mb-4">
              <div class="w-12 h-1 bg-muted-foreground/30 rounded-full mx-auto" />
            </div>
            <h2 class="text-lg font-semibold mb-4">AI Prompt</h2>
            <textarea
              class="w-full p-3 bg-secondary rounded-xl resize-none"
              placeholder="Describe what you want to change..."
              rows="3"
            />
            <div class="flex gap-2 mt-4">
              <button class="btn btn-secondary flex-1">
                🎤 Voice
              </button>
              <button class="btn btn-primary flex-1">
                ⚡ Generate
              </button>
            </div>
          </div>
        </Show>
      </div>
    </>
  );
}