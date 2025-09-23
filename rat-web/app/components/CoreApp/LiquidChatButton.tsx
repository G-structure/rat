import { createSignal, Show, onMount, onCleanup, createEffect } from "solid-js";
import { LiquidChatModal } from "./LiquidChatModal";

interface LiquidChatButtonProps {
  selectedText?: string;
  currentFile?: string;
  repoName?: string;
}

export function LiquidChatButton(props: LiquidChatButtonProps) {
  const [isModalOpen, setIsModalOpen] = createSignal(false);
  const [hasNewMessages, setHasNewMessages] = createSignal(false);
  
  
  // Auto-open when text is selected (optional behavior)
  createEffect(() => {
    if (props.selectedText && props.selectedText.length > 10) {
      // Uncomment the line below to auto-open chat when text is selected
      // setIsModalOpen(true);
      setHasNewMessages(true); // Just indicate new content available
    }
  });
  
  const handleOpenModal = () => {
    setIsModalOpen(true);
    setHasNewMessages(false);
  };
  
  // Keyboard shortcut handler
  onMount(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd/Ctrl + / to toggle chat
      if ((e.metaKey || e.ctrlKey) && e.key === "/") {
        e.preventDefault();
        setIsModalOpen(!isModalOpen());
      }
    };
    
    document.addEventListener("keydown", handleKeyDown);
    onCleanup(() => document.removeEventListener("keydown", handleKeyDown));
  });
  
  return (
    <>
      {/* Floating Action Button */}
      <button
        onClick={handleOpenModal}
        class="fixed bottom-6 right-6 w-14 h-14 rounded-full shadow-2xl flex items-center justify-center transition-all duration-300 hover:scale-105 active:scale-95 z-[9999] bg-gradient-to-br from-blue-500 via-purple-500 to-pink-500 hover:shadow-[0_0_20px_rgba(168,85,247,0.5)]"
        title="Open AI Chat (⌘/)"
        style="position: fixed !important; bottom: 24px !important; right: 24px !important;"
      >
        <Show when={hasNewMessages}>
          <div class="absolute -top-1 -right-1 w-3 h-3 bg-green-500 rounded-full animate-pulse shadow-[0_0_8px_rgba(34,197,94,0.6)]" />
        </Show>
        <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z" />
        </svg>
        
        {/* Ripple effect on hover */}
        <div class="absolute inset-0 rounded-full bg-white/20 scale-0 group-hover:scale-100 transition-transform duration-300" />
      </button>
      
      {/* Liquid Chat Modal */}
      <Show when={isModalOpen()}>
        <LiquidChatModal 
          isOpen={isModalOpen()} 
          onClose={() => setIsModalOpen(false)}
          selectedText={props.selectedText}
          currentFile={props.currentFile}
          repoName={props.repoName}
        />
      </Show>
    </>
  );
}