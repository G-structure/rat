import { createSignal, createMemo, Show, For } from "solid-js";
import { useParams, useNavigate } from "@solidjs/router";
import { createQuery } from "@tanstack/solid-query";
import { Title } from "@solidjs/meta";

export default function RepoView() {
  const params = useParams();
  const navigate = useNavigate();
  const [showPromptSheet, setShowPromptSheet] = createSignal(false);
  
  // Parse the slug to get owner, repo, and file path
  const pathInfo = createMemo(() => {
    const slug = params.slug;
    if (!slug) return null;
    
    const parts = slug.split("/");
    if (parts.length < 2) return null;
    
    return {
      owner: parts[0],
      repo: parts[1],
      branch: parts[2] || "main",
      filePath: parts.slice(3).join("/") || ""
    };
  });
  
  // Fetch repository data
  const repoQuery = createQuery(() => ({
    queryKey: ["repo", pathInfo()?.owner, pathInfo()?.repo],
    queryFn: async () => {
      const info = pathInfo();
      if (!info) throw new Error("Invalid path");
      
      const response = await fetch(`/api/github/repos/${info.owner}/${info.repo}`);
      if (!response.ok) throw new Error("Failed to fetch repository");
      return response.json();
    },
    enabled: !!pathInfo()
  }));
  
  // Fetch file/directory contents
  const contentsQuery = createQuery(() => ({
    queryKey: ["contents", pathInfo()?.owner, pathInfo()?.repo, pathInfo()?.branch, pathInfo()?.filePath],
    queryFn: async () => {
      const info = pathInfo();
      if (!info) throw new Error("Invalid path");
      
      const path = info.filePath || "";
      const response = await fetch(
        `/api/github/files/${info.owner}/${info.repo}/${info.branch}/${path}`
      );
      if (!response.ok) throw new Error("Failed to fetch contents");
      return response.json();
    },
    enabled: !!pathInfo()
  }));
  
  const isFile = createMemo(() => {
    const data = contentsQuery.data;
    return data && !Array.isArray(data);
  });
  
  const breadcrumbs = createMemo(() => {
    const info = pathInfo();
    if (!info) return [];
    
    const parts = [
      { name: info.repo, path: `/repos/${info.owner}/${info.repo}` }
    ];
    
    if (info.filePath) {
      const pathParts = info.filePath.split("/");
      pathParts.forEach((part, index) => {
        const path = `/repos/${info.owner}/${info.repo}/${info.branch}/${pathParts.slice(0, index + 1).join("/")}`;
        parts.push({ name: part, path });
      });
    }
    
    return parts;
  });
  
  function navigateToFile(item: any) {
    const info = pathInfo();
    if (!info) return;
    
    const newPath = info.filePath 
      ? `${info.filePath}/${item.name}`
      : item.name;
    
    navigate(`/repos/${info.owner}/${info.repo}/${info.branch}/${newPath}`);
  }
  
  return (
    <>
      <Title>{pathInfo()?.filePath || pathInfo()?.repo || "Repository"} - RAT Mobile IDE</Title>
      <div class="h-[100dvh] flex flex-col bg-background">
        {/* Header */}
        <header class="border-b border-border safe-top">
          <div class="p-4">
            <button 
              onClick={() => navigate("/dashboard")}
              class="flex items-center gap-2 text-muted-foreground hover:text-foreground mb-3"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
              </svg>
              <span>Dashboard</span>
            </button>
            
            {/* Breadcrumbs */}
            <div class="flex items-center gap-1 text-sm overflow-x-auto scrollbar-none">
              <For each={breadcrumbs()}>
                {(crumb, index) => (
                  <>
                    <Show when={index() > 0}>
                      <span class="text-muted-foreground">/</span>
                    </Show>
                    <button
                      onClick={() => navigate(crumb.path)}
                      class="font-medium hover:text-primary whitespace-nowrap"
                    >
                      {crumb.name}
                    </button>
                  </>
                )}
              </For>
            </div>
          </div>
          
          {/* Branch selector */}
          <div class="px-4 pb-3 flex items-center gap-2">
            <button class="px-3 py-1.5 text-sm bg-secondary rounded-lg flex items-center gap-2">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
              </svg>
              {pathInfo()?.branch}
            </button>
          </div>
        </header>
        
        {/* Content area */}
        <div class="flex-1 overflow-hidden">
          <Show when={contentsQuery.isLoading}>
            <div class="flex items-center justify-center h-full">
              <div class="text-muted-foreground">Loading...</div>
            </div>
          </Show>
          
          <Show when={contentsQuery.error}>
            <div class="flex items-center justify-center h-full p-4">
              <div class="text-center">
                <p class="text-destructive font-medium">Error loading content</p>
                <p class="text-sm text-muted-foreground mt-1">
                  {contentsQuery.error?.message}
                </p>
              </div>
            </div>
          </Show>
          
          <Show when={!isFile() && contentsQuery.data}>
            {/* Directory listing */}
            <div class="overflow-y-auto h-full">
              <div class="p-4 space-y-2">
                <For each={contentsQuery.data as any[]}>
                  {(item) => (
                    <button
                      onClick={() => navigateToFile(item)}
                      class="w-full p-3 rounded-xl bg-secondary/50 hover:bg-secondary transition-colors text-left flex items-center gap-3"
                    >
                      <Show when={item.type === "dir"}>
                        <svg class="w-5 h-5 text-blue-500" fill="currentColor" viewBox="0 0 24 24">
                          <path d="M10 4H4c-1.11 0-2 .89-2 2v12c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V8c0-1.11-.89-2-2-2h-8l-2-2z"/>
                        </svg>
                      </Show>
                      <Show when={item.type === "file"}>
                        <svg class="w-5 h-5 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                        </svg>
                      </Show>
                      <div class="flex-1 min-w-0">
                        <p class="font-medium truncate">{item.name}</p>
                        <Show when={item.size !== undefined}>
                          <p class="text-xs text-muted-foreground">
                            {item.type === "file" ? `${(item.size / 1024).toFixed(1)} KB` : "Directory"}
                          </p>
                        </Show>
                      </div>
                      <svg class="w-5 h-5 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                      </svg>
                    </button>
                  )}
                </For>
              </div>
            </div>
          </Show>
          
          <Show when={isFile() && contentsQuery.data}>
            {/* File viewer (placeholder for CodeMirror) */}
            <div class="h-full flex flex-col">
              <div class="flex-1 p-4 overflow-auto">
                <pre class="text-sm font-mono whitespace-pre-wrap">
                  {contentsQuery.data.content}
                </pre>
              </div>
            </div>
          </Show>
        </div>
        
        {/* Floating action button for files */}
        <Show when={isFile()}>
          <button
            onClick={() => setShowPromptSheet(true)}
            class="fixed bottom-6 right-6 w-14 h-14 rounded-full bg-primary text-primary-foreground shadow-lg flex items-center justify-center safe"
          >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
            </svg>
          </button>
        </Show>
        
        {/* Prompt sheet (reusable component needed) */}
        <Show when={showPromptSheet()}>
          <div 
            class="fixed inset-0 bg-black/50 z-40"
            onClick={() => setShowPromptSheet(false)}
          />
          <div class="fixed inset-x-0 bottom-0 bg-background border-t border-border rounded-t-3xl p-6 safe animate-slide-up z-50">
            <div class="drag-handle mb-4">
              <div class="w-12 h-1 bg-muted-foreground/30 rounded-full mx-auto" />
            </div>
            <h2 class="text-lg font-semibold mb-4">Edit with AI</h2>
            <textarea
              class="w-full p-3 bg-secondary rounded-xl resize-none"
              placeholder="Describe the changes you want to make..."
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