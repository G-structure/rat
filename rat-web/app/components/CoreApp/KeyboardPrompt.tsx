import { createSignal, onMount, Show } from "solid-js";
import { Portal } from "solid-js/web";

interface KeyboardPromptProps {
  selectedText: string;
  chatHistory?: Array<{prompt: string; response?: string; timestamp: number}>;
  onClose: () => void;
  onSubmit: (prompt: string) => void;
}

export function KeyboardPrompt(props: KeyboardPromptProps) {
  const [prompt, setPrompt] = createSignal("");
  const [showKeyboard, setShowKeyboard] = createSignal(false);
  let inputRef: HTMLInputElement | undefined;
  
  onMount(() => {
    inputRef?.focus();
    // Show keyboard UI on mobile
    if (window.innerWidth < 768) {
      setShowKeyboard(true);
    }
  });
  
  const handleSubmit = () => {
    if (prompt().trim()) {
      props.onSubmit(prompt());
      props.onClose();
    }
  };
  
  const quickActions = [
    { label: "Refactor", action: "Refactor this code" },
    { label: "Fix", action: "Fix any issues" },
    { label: "Explain", action: "Explain this code" },
    { label: "Test", action: "Write tests for this" },
    { label: "Type", action: "Add TypeScript types" },
    { label: "Comment", action: "Add comments" }
  ];
  
  return (
    <Portal>
      <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
        {/* Backdrop */}
        <div 
          class="absolute inset-0 bg-black/70 backdrop-blur-sm"
          onClick={props.onClose}
        />
        
        {/* Modal */}
        <div class="relative w-full max-w-md bg-background border border-border rounded-2xl shadow-2xl animate-scale-up">
          {/* Header */}
          <div class="p-4 border-b border-border">
            <div class="flex items-center justify-between mb-2">
              <h3 class="text-lg font-semibold">Quick Edit</h3>
              <kbd class="text-xs px-2 py-1 bg-secondary rounded">⌘K</kbd>
            </div>
            {props.selectedText && (
              <div class="text-sm text-muted-foreground">
                <span class="font-medium">Selected:</span>
                <pre class="mt-1 p-2 bg-secondary rounded text-xs overflow-x-auto">
                  {props.selectedText.substring(0, 100)}
                  {props.selectedText.length > 100 && "..."}
                </pre>
              </div>
            )}
          </div>
          
          {/* Input */}
          <div class="p-4 space-y-3">
            <div class="relative">
              <input
                ref={inputRef}
                type="text"
                value={prompt()}
                onInput={(e) => setPrompt(e.currentTarget.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    handleSubmit();
                  }
                }}
                placeholder="What do you want to do?"
                class="w-full px-4 py-3 bg-secondary rounded-xl focus:outline-none focus:ring-2 focus:ring-primary"
              />
              <button
                onClick={handleSubmit}
                disabled={!prompt().trim()}
                class="absolute right-2 top-1/2 -translate-y-1/2 p-2 text-primary disabled:opacity-50"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 9l3 3m0 0l-3 3m3-3H8m13 0a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </button>
            </div>
            
            {/* Quick Actions */}
            <div class="grid grid-cols-3 gap-2">
              {quickActions.map(({ label, action }) => (
                <button
                  onClick={() => {
                    setPrompt(action);
                    inputRef?.focus();
                  }}
                  class="px-3 py-2 text-sm bg-secondary hover:bg-secondary/80 rounded-lg transition-colors"
                >
                  {label}
                </button>
              ))}
            </div>
          </div>
          
          {/* Mobile Keyboard UI */}
          <Show when={showKeyboard()}>
            <div class="border-t border-border p-4 bg-secondary/50">
              <div class="grid grid-cols-10 gap-1 text-sm">
                {["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"].map(key => (
                  <button
                    onClick={() => setPrompt(prompt() + key)}
                    class="p-2 bg-background hover:bg-secondary rounded"
                  >
                    {key}
                  </button>
                ))}
              </div>
              <div class="grid grid-cols-9 gap-1 mt-1 text-sm mx-4">
                {["a", "s", "d", "f", "g", "h", "j", "k", "l"].map(key => (
                  <button
                    onClick={() => setPrompt(prompt() + key)}
                    class="p-2 bg-background hover:bg-secondary rounded"
                  >
                    {key}
                  </button>
                ))}
              </div>
              <div class="flex gap-1 mt-1 text-sm">
                <button
                  onClick={() => setPrompt(prompt().slice(0, -1))}
                  class="flex-1 p-2 bg-background hover:bg-secondary rounded"
                >
                  ←
                </button>
                {["z", "x", "c", "v", "b", "n", "m"].map(key => (
                  <button
                    onClick={() => setPrompt(prompt() + key)}
                    class="p-2 bg-background hover:bg-secondary rounded"
                  >
                    {key}
                  </button>
                ))}
                <button
                  onClick={handleSubmit}
                  class="flex-1 p-2 bg-primary text-primary-foreground rounded"
                >
                  Go
                </button>
              </div>
              <button
                onClick={() => setPrompt(prompt() + " ")}
                class="w-full mt-1 p-3 bg-background hover:bg-secondary rounded"
              >
                space
              </button>
            </div>
          </Show>
        </div>
      </div>
    </Portal>
  );
}