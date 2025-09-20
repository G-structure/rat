import { createSignal, Show, onMount, onCleanup, createMemo } from "solid-js";
import { Title } from "@solidjs/meta";
import { useSearchParams } from "@solidjs/router";
import { Sidebar } from "~/components/CoreApp/Sidebar";
import { CodeEditor } from "~/components/CoreApp/CodeEditor";
import { ChatBubble } from "~/components/CoreApp/ChatBubble";
import { ChatModal } from "~/components/CoreApp/ChatModal";
import { KeyboardPrompt } from "~/components/CoreApp/KeyboardPrompt";
import { CodeReviewPanel } from "~/components/CoreApp/CodeReviewPanel";
import { useLocalStorage } from "~/hooks/useLocalStorage";
import { showToast } from "~/components/Common/Toast";

export default function CoreApp() {
  const [searchParams] = useSearchParams();
  const repoName = searchParams.repo || "default-project";
  const [loadingState, setLoadingState] = createSignal("idle");
  
  const [sidebarOpen, setSidebarOpen] = createSignal(true);
  const [chatModalOpen, setChatModalOpen] = createSignal(false);
  const [keyboardPromptOpen, setKeyboardPromptOpen] = createSignal(false);
  const [reviewPanelOpen, setReviewPanelOpen] = createSignal(false);
  const [selectedFile, setSelectedFile] = createSignal<string | null>("src/App.tsx");
  const [chatContent, setChatContent] = useLocalStorage("chat-content", "");
  const [selectedText, setSelectedText] = createSignal("");
  const [isGenerating, setIsGenerating] = createSignal(false);
  
  // Mock file changes for review
  const [filesAffected] = createSignal([
    { path: "src/App.tsx", additions: 15, deletions: 3, status: "modified" },
    { path: "src/components/Button.tsx", additions: 42, deletions: 0, status: "added" },
    { path: "src/utils/helpers.ts", additions: 5, deletions: 8, status: "modified" },
    { path: "src/old-file.ts", additions: 0, deletions: 25, status: "deleted" }
  ]);

  // Handle keyboard shortcuts
  onMount(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd/Ctrl + K for keyboard prompt
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setKeyboardPromptOpen(true);
      }
      
      // Cmd/Ctrl + / for chat toggle
      if ((e.metaKey || e.ctrlKey) && e.key === "/") {
        e.preventDefault();
        setChatModalOpen(!chatModalOpen());
      }
      
      // Cmd/Ctrl + Shift + R for review panel
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === "R") {
        e.preventDefault();
        setReviewPanelOpen(!reviewPanelOpen());
      }
      
      // Escape to close modals
      if (e.key === "Escape") {
        if (keyboardPromptOpen()) setKeyboardPromptOpen(false);
        else if (chatModalOpen()) setChatModalOpen(false);
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
          <div class="h-12 border-b border-border flex items-center justify-between px-4">
            <div class="flex items-center gap-4">
              <Show when={!sidebarOpen()}>
                <button
                  onClick={() => setSidebarOpen(true)}
                  class="p-2 hover:bg-secondary rounded-lg"
                >
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
                  </svg>
                </button>
              </Show>
              <span class="text-sm text-muted-foreground">{selectedFile()}</span>
            </div>
            
            <div class="flex items-center gap-2">
              <button
                onClick={() => setReviewPanelOpen(true)}
                class="px-3 py-1.5 text-sm bg-secondary hover:bg-secondary/80 rounded-lg flex items-center gap-2"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
                </svg>
                Review Changes
              </button>
            </div>
          </div>
          
          {/* Code Editor */}
          <div class="flex-1 overflow-hidden">
            <CodeEditor 
              file={selectedFile()}
              onTextSelect={handleTextSelection}
            />
          </div>
        </div>
        
        {/* Chat Bubble - Always visible */}
        <Show when={!chatModalOpen()}>
          <ChatBubble 
            onClick={() => setChatModalOpen(true)}
            hasContent={chatContent().length > 0}
          />
        </Show>
        
        {/* Chat Modal */}
        <Show when={chatModalOpen()}>
          <ChatModal
            content={chatContent()}
            onContentChange={setChatContent}
            onClose={() => setChatModalOpen(false)}
          />
        </Show>
        
        {/* Keyboard Prompt Modal */}
        <Show when={keyboardPromptOpen()}>
          <KeyboardPrompt
            selectedText={selectedText()}
            onClose={() => setKeyboardPromptOpen(false)}
            onSubmit={async (prompt) => {
              console.log("Prompt submitted:", prompt);
              setKeyboardPromptOpen(false);
              setLoadingState("generating");
              showToast("Generating code changes...", "info");
              
              // Simulate AI generation
              await new Promise(resolve => setTimeout(resolve, 2000));
              
              setLoadingState("idle");
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