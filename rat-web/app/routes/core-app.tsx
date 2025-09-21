import { createSignal, Show, onMount, onCleanup, createEffect } from "solid-js";
import { Title } from "@solidjs/meta";
import { useSearchParams, useNavigate } from "@solidjs/router";
import { Sidebar } from "~/components/CoreApp/Sidebar";
import { CodeEditor } from "~/components/CoreApp/CodeEditor";
import { LiquidChatButton } from "~/components/CoreApp/LiquidChatButton";
import { KeyboardPrompt } from "~/components/CoreApp/KeyboardPrompt";
import { CodeReviewPanel } from "~/components/CoreApp/CodeReviewPanel";
import { showToast } from "~/components/Common/Toast";
import { chatStore } from "~/stores/chatStore";
import { editorStore } from "~/stores/editorStore";

export default function CoreApp() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const repoName = searchParams.repo || "default-project";
  const [setLoadingState] = createSignal("idle");
  
  const [sidebarOpen, setSidebarOpen] = createSignal(true);
  const [keyboardPromptOpen, setKeyboardPromptOpen] = createSignal(false);
  const [reviewPanelOpen, setReviewPanelOpen] = createSignal(false);
  const [selectedFile, setSelectedFile] = createSignal<string | null>("src/App.tsx");
  const [selectedText, setSelectedText] = createSignal("");
  const [chatHistory, setChatHistory] = createSignal<Array<{prompt: string; response?: string; timestamp: number}>>([]);
  
  // Mock file changes for review
  const [filesAffected] = createSignal([
    { path: "src/App.tsx", additions: 15, deletions: 3, status: "modified" as const },
    { path: "src/components/Button.tsx", additions: 42, deletions: 0, status: "added" as const },
    { path: "src/utils/helpers.ts", additions: 5, deletions: 8, status: "modified" as const },
    { path: "src/old-file.ts", additions: 0, deletions: 25, status: "deleted" as const }
  ]);

  // Load chat history on mount
  onMount(() => {
    const history = chatStore.getProjectHistory(repoName);
    if (history) {
      setChatHistory(history.prompts);
    }
  });

  // Handle keyboard shortcuts
  onMount(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd/Ctrl + K for keyboard prompt
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setKeyboardPromptOpen(true);
      }
      
      // Cmd/Ctrl + Z for undo
      if ((e.metaKey || e.ctrlKey) && !e.shiftKey && e.key === "z") {
        e.preventDefault();
        if (editorStore.canUndo()) {
          editorStore.undo();
          showToast("Undo", "success");
        }
      }
      
      // Cmd/Ctrl + Shift + Z for redo
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === "z") {
        e.preventDefault();
        if (editorStore.canRedo()) {
          editorStore.redo();
          showToast("Redo", "success");
        }
      }
      
      // Cmd/Ctrl + Shift + R for review panel
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === "R") {
        e.preventDefault();
        setReviewPanelOpen(!reviewPanelOpen());
      }
      
      // Escape to close modals
      if (e.key === "Escape") {
        if (keyboardPromptOpen()) setKeyboardPromptOpen(false);
        else if (reviewPanelOpen()) setReviewPanelOpen(false);
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    onCleanup(() => document.removeEventListener("keydown", handleKeyDown));
  });

  const handleTextSelection = (text: string) => {
    if (text) {
      setSelectedText(text);
      // Auto-open keyboard prompt when text is selected
      setTimeout(() => {
        if (text.length > 5) {
          setKeyboardPromptOpen(true);
        }
      }, 500);
    }
  };

  return (
    <>
      <Title>RAT IDE - Code Editor</Title>
      
      <div class="h-[100dvh] flex bg-background text-foreground overflow-hidden">
        {/* Sidebar */}
        <Sidebar 
          open={sidebarOpen()} 
          onToggle={() => setSidebarOpen(!sidebarOpen())}
          selectedFile={selectedFile()}
          onFileSelect={setSelectedFile}
          repoName={repoName}
        />
        
        {/* Main Editor Area */}
        <div class="flex-1 flex flex-col">
          {/* Editor Header */}
          <div class="h-12 border-b border-border grid grid-cols-3 items-center px-4">
            {/* Left section */}
            <div class="flex items-center gap-2">
              <Show when={!sidebarOpen()}>
                <button
                  onClick={() => setSidebarOpen(true)}
                  class="p-2 hover:bg-secondary rounded-lg"
                  title="Open Sidebar"
                >
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
                  </svg>
                </button>
              </Show>
              <button
                onClick={() => navigate('/dashboard')}
                class="p-2 hover:bg-secondary rounded-lg"
                title="Home"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
                </svg>
              </button>
              <span class="text-sm text-muted-foreground ml-2">{selectedFile()}</span>
            </div>
            
            {/* Center section - Undo/Redo */}
            <div class="flex items-center justify-center gap-1">
              <button
                onClick={() => {
                  editorStore.undo();
                  showToast("Undo", "success");
                }}
                class="p-2 hover:bg-secondary rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
                title="Undo (Ctrl+Z)"
                disabled={!editorStore.canUndo()}
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6" />
                </svg>
              </button>
              <button
                onClick={() => {
                  editorStore.redo();
                  showToast("Redo", "success");
                }}
                class="p-2 hover:bg-secondary rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
                title="Redo (Ctrl+Shift+Z)"
                disabled={!editorStore.canRedo()}
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 10H11a8 8 0 00-8 8v2m18-10l-6 6m6-6l-6-6" />
                </svg>
              </button>
            </div>
            
            {/* Right section */}
            <div class="flex items-center justify-end gap-2">
              <button
                onClick={() => setReviewPanelOpen(true)}
                class="px-3 py-1.5 text-sm bg-secondary hover:bg-secondary/80 rounded-lg"
              >
                Review Changes
              </button>
            </div>
          </div>
          
          {/* Code Editor */}
          <div class="flex-1 overflow-hidden">
            <CodeEditor 
              file={selectedFile()}
              onTextSelect={handleTextSelection}
              repoName={repoName}
            />
          </div>
        </div>
        
        {/* Liquid Chat Button */}
        <LiquidChatButton />
        
        {/* Keyboard Prompt Modal */}
        <Show when={keyboardPromptOpen()}>
          <KeyboardPrompt
            selectedText={selectedText()}
            chatHistory={chatHistory()}
            onClose={() => setKeyboardPromptOpen(false)}
            onSubmit={async (prompt) => {
              console.log("Prompt submitted:", prompt);
              setKeyboardPromptOpen(false);
              setLoadingState("generating");
              
              // Update chat state for this project
              chatStore.updateProjectState(repoName, "generating", prompt);
              chatStore.addPromptToHistory(repoName, prompt);
              
              showToast("Generating code changes...", "info");
              
              // Simulate AI generation
              await new Promise(resolve => setTimeout(resolve, 2000));
              
              setLoadingState("idle");
              chatStore.updateProjectState(repoName, "completed", prompt);
              chatStore.addPromptToHistory(repoName, prompt, "Generated code successfully");
              
              // Reset to idle after 5 seconds
              setTimeout(() => {
                chatStore.updateProjectState(repoName, "idle");
              }, 5000);
              
              showToast("Code generated successfully!", "success");
              setReviewPanelOpen(true); // Show review panel after generation
            }}
          />
        </Show>
        
        {/* Code Review Panel */}
        <Show when={reviewPanelOpen()}>
          <CodeReviewPanel
            filesAffected={filesAffected()}
            onClose={() => setReviewPanelOpen(false)}
          />
        </Show>
      </div>
    </>
  );
}