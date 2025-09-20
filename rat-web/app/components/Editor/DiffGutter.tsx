import { For, Show, createSignal } from "solid-js";
import { Portal } from "solid-js/web";

interface DiffHunk {
  type: "add" | "remove" | "modify";
  startLine: number;
  endLine: number;
  content: string;
  originalContent?: string;
}

interface DiffGutterProps {
  hunks: DiffHunk[];
  onAccept: (hunk: DiffHunk) => void;
  onReject: (hunk: DiffHunk) => void;
  onAcceptAll: () => void;
  onRejectAll: () => void;
  currentLine?: number;
}

export function DiffGutter(props: DiffGutterProps) {
  const [selectedHunk, setSelectedHunk] = createSignal<DiffHunk | null>(null);
  const [showPreview, setShowPreview] = createSignal(false);
  
  const getGutterColor = (type: DiffHunk["type"]) => {
    switch (type) {
      case "add":
        return "bg-green-500";
      case "remove":
        return "bg-red-500";
      case "modify":
        return "bg-yellow-500";
    }
  };
  
  const getGutterIcon = (type: DiffHunk["type"]) => {
    switch (type) {
      case "add":
        return "+";
      case "remove":
        return "-";
      case "modify":
        return "~";
    }
  };
  
  const handleHunkClick = (hunk: DiffHunk) => {
    setSelectedHunk(hunk);
    setShowPreview(true);
  };
  
  return (
    <>
      {/* Gutter indicators */}
      <div class="absolute left-0 top-0 w-1 h-full pointer-events-none">
        <For each={props.hunks}>
          {(hunk) => {
            const height = (hunk.endLine - hunk.startLine + 1) * 20; // Approximate line height
            const top = hunk.startLine * 20;
            
            return (
              <button
                onClick={() => handleHunkClick(hunk)}
                class={`absolute left-0 w-1 hover:w-2 transition-all cursor-pointer pointer-events-auto ${getGutterColor(hunk.type)}`}
                style={{
                  top: `${top}px`,
                  height: `${height}px`
                }}
                title={`Lines ${hunk.startLine}-${hunk.endLine}: ${hunk.type}`}
              />
            );
          }}
        </For>
      </div>
      
      {/* Summary bar */}
      <Show when={props.hunks.length > 0}>
        <div class="absolute top-2 right-2 flex items-center gap-2 px-3 py-1.5 bg-background/90 backdrop-blur-sm rounded-lg border border-border shadow-sm">
          <span class="text-xs text-muted-foreground">
            {props.hunks.length} changes
          </span>
          <div class="flex items-center gap-1">
            <button
              onClick={props.onAcceptAll}
              class="p-1 rounded hover:bg-green-500/20 text-green-500 transition-colors"
              title="Accept all changes"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
            </button>
            <button
              onClick={props.onRejectAll}
              class="p-1 rounded hover:bg-red-500/20 text-red-500 transition-colors"
              title="Reject all changes"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>
      </Show>
      
      {/* Hunk preview popup */}
      <Show when={showPreview() && selectedHunk()}>
        <Portal>
          <div 
            class="fixed inset-0 bg-black/50 z-40"
            onClick={() => setShowPreview(false)}
          />
          <div class="fixed inset-x-4 top-1/4 max-w-lg mx-auto bg-background border border-border rounded-xl shadow-xl z-50 animate-fade-in">
            <div class="p-4 border-b border-border">
              <div class="flex items-center justify-between">
                <h3 class="font-semibold">
                  {selectedHunk()!.type === "add" && "Addition"}
                  {selectedHunk()!.type === "remove" && "Removal"}
                  {selectedHunk()!.type === "modify" && "Modification"}
                </h3>
                <span class="text-sm text-muted-foreground">
                  Lines {selectedHunk()!.startLine}-{selectedHunk()!.endLine}
                </span>
              </div>
            </div>
            
            <div class="p-4 space-y-3 max-h-64 overflow-y-auto">
              <Show when={selectedHunk()!.originalContent}>
                <div>
                  <p class="text-xs font-medium text-muted-foreground mb-1">Original:</p>
                  <pre class="p-2 rounded bg-red-500/10 text-sm font-mono overflow-x-auto">
                    {selectedHunk()!.originalContent}
                  </pre>
                </div>
              </Show>
              
              <Show when={selectedHunk()!.type !== "remove"}>
                <div>
                  <p class="text-xs font-medium text-muted-foreground mb-1">New:</p>
                  <pre class="p-2 rounded bg-green-500/10 text-sm font-mono overflow-x-auto">
                    {selectedHunk()!.content}
                  </pre>
                </div>
              </Show>
            </div>
            
            <div class="p-4 border-t border-border flex gap-2">
              <button
                onClick={() => {
                  props.onAccept(selectedHunk()!);
                  setShowPreview(false);
                }}
                class="btn btn-primary flex-1 flex items-center justify-center gap-2"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
                Accept
              </button>
              <button
                onClick={() => {
                  props.onReject(selectedHunk()!);
                  setShowPreview(false);
                }}
                class="btn btn-secondary flex-1 flex items-center justify-center gap-2"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
                Reject
              </button>
            </div>
          </div>
        </Portal>
      </Show>
    </>
  );
}