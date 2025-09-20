import { createSignal, Show } from "solid-js";
import { BottomSheet } from "../Mobile/BottomSheet";

interface PromptSheetProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (prompt: string) => void;
  selectedFile?: string;
  selectedText?: string;
}

export function PromptSheet(props: PromptSheetProps) {
  const [prompt, setPrompt] = createSignal("");
  const [isRecording, setIsRecording] = createSignal(false);
  const [useVoice, setUseVoice] = createSignal(false);
  
  const handleSubmit = () => {
    const trimmedPrompt = prompt().trim();
    if (trimmedPrompt) {
      props.onSubmit(trimmedPrompt);
      setPrompt("");
      props.onClose();
    }
  };
  
  const handleVoiceToggle = () => {
    setUseVoice(!useVoice());
    if (!useVoice() && isRecording()) {
      setIsRecording(false);
    }
  };
  
  const handleRecordToggle = async () => {
    if (!useVoice()) return;
    
    if (isRecording()) {
      setIsRecording(false);
      // Stop recording logic here
    } else {
      setIsRecording(true);
      // Start recording logic here
      // For now, just simulate with a timeout
      setTimeout(() => {
        setIsRecording(false);
        setPrompt("Refactor this function to use async/await");
      }, 2000);
    }
  };
  
  return (
    <BottomSheet
      isOpen={props.isOpen}
      onClose={props.onClose}
      title="AI Assistant"
      snapPoints={[40, 75]}
      defaultSnap={0}
    >
      <div class="space-y-4">
        {/* Context display */}
        <Show when={props.selectedFile || props.selectedText}>
          <div class="p-3 rounded-xl bg-secondary/50 text-sm">
            <Show when={props.selectedFile}>
              <div class="flex items-center gap-2 text-muted-foreground mb-1">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
                <span>{props.selectedFile}</span>
              </div>
            </Show>
            <Show when={props.selectedText}>
              <div class="font-mono text-xs truncate">
                {props.selectedText}
              </div>
            </Show>
          </div>
        </Show>
        
        {/* Input area */}
        <div class="space-y-3">
          <Show when={!useVoice()}>
            <textarea
              value={prompt()}
              onInput={(e) => setPrompt(e.currentTarget.value)}
              placeholder="Describe what you want to change..."
              class="w-full p-3 bg-secondary rounded-xl resize-none focus:outline-none focus:ring-2 focus:ring-primary"
              rows="3"
              onKeyDown={(e) => {
                if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
                  handleSubmit();
                }
              }}
            />
          </Show>
          
          <Show when={useVoice()}>
            <div class="flex flex-col items-center justify-center p-8 bg-secondary rounded-xl">
              <button
                onClick={handleRecordToggle}
                class={`w-20 h-20 rounded-full flex items-center justify-center transition-all ${
                  isRecording() 
                    ? "bg-red-500 animate-pulse" 
                    : "bg-primary hover:bg-primary/90"
                }`}
              >
                <svg class="w-8 h-8 text-white" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M12 14c1.66 0 2.99-1.34 2.99-3L15 5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3zm5.3-3c0 3-2.54 5.1-5.3 5.1S6.7 14 6.7 11H5c0 3.41 2.72 6.23 6 6.72V21h2v-3.28c3.28-.48 6-3.3 6-6.72h-1.7z"/>
                </svg>
              </button>
              <p class="mt-4 text-sm text-muted-foreground">
                {isRecording() ? "Listening..." : "Tap to speak"}
              </p>
              <Show when={prompt()}>
                <p class="mt-4 text-sm italic">"{prompt()}"</p>
              </Show>
            </div>
          </Show>
        </div>
        
        {/* Quick prompts */}
        <div class="space-y-2">
          <p class="text-xs font-medium text-muted-foreground">Quick prompts:</p>
          <div class="flex flex-wrap gap-2">
            {[
              "Refactor this",
              "Add error handling", 
              "Write tests",
              "Add comments",
              "Fix lint issues"
            ].map(quickPrompt => (
              <button
                onClick={() => setPrompt(quickPrompt)}
                class="px-3 py-1 text-sm bg-secondary rounded-full hover:bg-secondary/80"
              >
                {quickPrompt}
              </button>
            ))}
          </div>
        </div>
        
        {/* Actions */}
        <div class="flex gap-2">
          <button
            onClick={handleVoiceToggle}
            class={`btn flex-1 flex items-center justify-center gap-2 ${
              useVoice() ? "btn-primary" : "btn-secondary"
            }`}
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" />
            </svg>
            {useVoice() ? "Voice" : "Text"}
          </button>
          <button
            onClick={handleSubmit}
            disabled={!prompt().trim()}
            class="btn btn-primary flex-1 flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
            Generate
          </button>
        </div>
        
        <p class="text-xs text-center text-muted-foreground">
          Press <kbd class="px-1 py-0.5 text-xs bg-secondary rounded">⌘</kbd> + <kbd class="px-1 py-0.5 text-xs bg-secondary rounded">Enter</kbd> to submit
        </p>
      </div>
    </BottomSheet>
  );
}