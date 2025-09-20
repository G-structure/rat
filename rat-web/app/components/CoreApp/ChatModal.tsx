import { createSignal, onMount, Show } from "solid-js";
import { Portal } from "solid-js/web";
import { showToast } from "~/components/Common/Toast";

interface ChatModalProps {
  content: string;
  onContentChange: (content: string) => void;
  onClose: () => void;
}

export function ChatModal(props: ChatModalProps) {
  const [isRecording, setIsRecording] = createSignal(false);
  const [transcript, setTranscript] = createSignal("");
  let textareaRef: HTMLTextAreaElement | undefined;
  
  onMount(() => {
    textareaRef?.focus();
    // Auto-resize textarea
    if (textareaRef) {
      textareaRef.style.height = "auto";
      textareaRef.style.height = textareaRef.scrollHeight + "px";
    }
  });
  
  const handleInput = (e: Event) => {
    const target = e.target as HTMLTextAreaElement;
    props.onContentChange(target.value);
    // Auto-resize
    target.style.height = "auto";
    target.style.height = target.scrollHeight + "px";
  };
  
  const handleVoiceRecord = async () => {
    if (!isRecording()) {
      setIsRecording(true);
      showToast("Recording started...", "info");
      // Mock voice recording
      setTimeout(() => {
        const mockTranscript = "Refactor this function to use modern JavaScript features and add error handling";
        setTranscript(mockTranscript);
        props.onContentChange(props.content + (props.content ? "\n" : "") + mockTranscript);
        setIsRecording(false);
        showToast("Recording stopped", "success");
      }, 2000);
    } else {
      setIsRecording(false);
    }
  };
  
  const handleSubmit = () => {
    if (props.content.trim()) {
      console.log("Submitting prompt:", props.content);
      showToast("Processing your request...", "info");
      // Simulate AI processing
      setTimeout(() => {
        showToast("AI response generated!", "success");
        props.onContentChange(props.content + "\n\n🤖 AI: I've analyzed your request. Here's what I suggest:\n\n1. Implement a custom hook for data fetching\n2. Add error boundaries for better error handling\n3. Optimize re-renders with memo and callbacks\n\nWould you like me to generate the code?");
      }, 1500);
    }
  };
  
  return (
    <Portal>
      <div class="fixed inset-0 z-50 flex items-end sm:items-center justify-center">
        {/* Backdrop */}
        <div 
          class="absolute inset-0 bg-black/50 backdrop-blur-sm"
          onClick={props.onClose}
        />
        
        {/* Modal */}
        <div class="relative w-full max-w-2xl mx-4 mb-4 sm:mb-0 bg-background border border-border rounded-2xl shadow-2xl animate-slide-up">
          {/* Header */}
          <div class="flex items-center justify-between p-4 border-b border-border">
            <h3 class="text-lg font-semibold">AI Assistant</h3>
            <button
              onClick={props.onClose}
              class="p-2 hover:bg-secondary rounded-lg transition-colors"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
          
          {/* Content */}
          <div class="p-4 space-y-4">
            <div class="relative">
              <textarea
                ref={textareaRef}
                value={props.content}
                onInput={handleInput}
                placeholder="Describe what you want to do..."
                class="w-full min-h-[120px] max-h-[400px] p-3 bg-secondary rounded-xl resize-none focus:outline-none focus:ring-2 focus:ring-primary"
                onKeyDown={(e) => {
                  if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
                    handleSubmit();
                  }
                }}
              />
              <div class="absolute bottom-2 right-2 text-xs text-muted-foreground">
                Press ⌘+Enter to submit
              </div>
            </div>
            
            {/* Voice Recording Indicator */}
            <Show when={isRecording()}>
              <div class="flex items-center gap-2 p-3 bg-red-500/10 rounded-lg animate-pulse">
                <div class="w-3 h-3 bg-red-500 rounded-full animate-pulse" />
                <span class="text-sm">Recording... Speak now</span>
              </div>
            </Show>
            
            {/* Actions */}
            <div class="flex gap-2">
              <button
                onClick={handleVoiceRecord}
                class={`btn flex-1 flex items-center justify-center gap-2 ${
                  isRecording() ? "bg-red-500 hover:bg-red-600 text-white" : "btn-secondary"
                }`}
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" />
                </svg>
                {isRecording() ? (
                  <>
                    <span class="animate-pulse">Stop Recording</span>
                  </>
                ) : (
                  "Voice Input"
                )}
              </button>
              
              <button
                onClick={handleSubmit}
                disabled={!props.content.trim()}
                class="btn btn-primary flex-1 flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
                Generate
              </button>
            </div>
            
            {/* Quick Actions */}
            <div class="border-t border-border pt-4">
              <p class="text-xs text-muted-foreground mb-2">Quick actions:</p>
              <div class="flex flex-wrap gap-2">
                {[
                  "Refactor this code",
                  "Add error handling",
                  "Write tests",
                  "Add TypeScript types",
                  "Optimize performance",
                  "Fix security issues",
                  "Improve accessibility",
                  "Add documentation"
                ].map(action => (
                  <button
                    onClick={() => props.onContentChange(action)}
                    class="px-3 py-1 text-sm bg-secondary hover:bg-secondary/80 rounded-full transition-colors"
                  >
                    {action}
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </Portal>
  );
}