import { createSignal, For, Show, onMount } from "solid-js";
import { Portal } from "solid-js/web";
import { showToast } from "~/components/Common/Toast";

interface FileChange {
  path: string;
  additions: number;
  deletions: number;
  status: "added" | "modified" | "deleted";
}

interface CodeReviewPanelProps {
  filesAffected: FileChange[];
  onClose: () => void;
}

export function CodeReviewPanel(props: CodeReviewPanelProps) {
  const [selectedFile, setSelectedFile] = createSignal<FileChange | null>(props.filesAffected[0]);
  const [expandedFiles, setExpandedFiles] = createSignal<Set<string>>(new Set());
  const [swipeX, setSwipeX] = createSignal(0);
  const [startX, setStartX] = createSignal(0);
  
  let panelRef: HTMLDivElement | undefined;
  
  // Mock code diffs
  const getDiff = (file: FileChange) => {
    if (file.status === "added") {
      return `+ import React from 'react';
+ 
+ export function ${file.path.split('/').pop()?.replace('.tsx', '')} {
+   const [state, setState] = useState(null);
+   
+   useEffect(() => {
+     // Initialize component
+     fetchData();
+   }, []);
+   
+   return (
+     <div className="component">
+       <h2>New Component</h2>
+       <p>This is a newly added component with hooks</p>
+       {state && <div>{state}</div>}
+     </div>
+   );
+ }`;
    } else if (file.status === "deleted") {
      return `- // This file has been deleted
- export function OldComponent() {
-   return <div>Removed</div>;
- }`;
    } else {
      return `  export function Component() {
    return (
      <div className="component">
-       <h2>Old Title</h2>
+       <h2>Updated Title</h2>
        <p>This component has been modified</p>
+       <button onClick={handleClick}>
+         New Button
+       </button>
      </div>
    );
  }`;
    }
  };
  
  const toggleFileExpanded = (path: string) => {
    setExpandedFiles(prev => {
      const next = new Set(prev);
      if (next.has(path)) {
        next.delete(path);
      } else {
        next.add(path);
      }
      return next;
    });
  };
  
  // Swipe handlers
  const handleTouchStart = (e: TouchEvent) => {
    setStartX(e.touches[0].clientX);
  };
  
  const handleTouchMove = (e: TouchEvent) => {
    const currentX = e.touches[0].clientX;
    const diff = currentX - startX();
    setSwipeX(diff);
  };
  
  const handleTouchEnd = () => {
    const threshold = 100;
    const currentIndex = props.filesAffected.findIndex(f => f === selectedFile());
    
    if (swipeX() > threshold && currentIndex > 0) {
      // Swipe right - previous file
      setSelectedFile(props.filesAffected[currentIndex - 1]);
    } else if (swipeX() < -threshold && currentIndex < props.filesAffected.length - 1) {
      // Swipe left - next file
      setSelectedFile(props.filesAffected[currentIndex + 1]);
    }
    
    setSwipeX(0);
  };
  
  onMount(() => {
    if (panelRef) {
      panelRef.addEventListener("touchstart", handleTouchStart);
      panelRef.addEventListener("touchmove", handleTouchMove);
      panelRef.addEventListener("touchend", handleTouchEnd);
    }
  });
  
  const getStatusColor = (status: string) => {
    switch (status) {
      case "added": return "text-green-500";
      case "modified": return "text-yellow-500";
      case "deleted": return "text-red-500";
      default: return "text-gray-500";
    }
  };
  
  const getStatusIcon = (status: string) => {
    switch (status) {
      case "added": return "+";
      case "modified": return "~";
      case "deleted": return "-";
      default: return "?";
    }
  };
  
  return (
    <Portal>
      <div class="fixed inset-0 z-50 flex">
        {/* Backdrop */}
        <div 
          class="absolute inset-0 bg-black/50 backdrop-blur-sm"
          onClick={props.onClose}
        />
        
        {/* Panel */}
        <div 
          ref={panelRef}
          class="relative ml-auto w-full max-w-2xl h-full bg-background border-l border-border shadow-2xl animate-slide-left overflow-hidden"
          style={{
            transform: `translateX(${swipeX()}px)`,
            transition: swipeX() === 0 ? "transform 0.3s" : "none"
          }}
        >
          {/* Header */}
          <div class="sticky top-0 bg-background border-b border-border p-4 z-10">
            <div class="flex items-center justify-between mb-4">
              <h2 class="text-xl font-bold">Review Changes</h2>
              <button
                onClick={props.onClose}
                class="p-2 hover:bg-secondary rounded-lg"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            
            <div class="flex items-center gap-4 text-sm">
              <span class="text-green-500">
                +{props.filesAffected.reduce((sum, f) => sum + f.additions, 0)} additions
              </span>
              <span class="text-red-500">
                -{props.filesAffected.reduce((sum, f) => sum + f.deletions, 0)} deletions
              </span>
              <span class="text-muted-foreground">
                {props.filesAffected.length} files
              </span>
            </div>
          </div>
          
          {/* Files List */}
          <div class="overflow-y-auto h-full pb-20">
            <div class="p-4 space-y-2">
              <For each={props.filesAffected}>
                {(file) => (
                  <div class="border border-border rounded-lg overflow-hidden">
                    <button
                      onClick={() => {
                        setSelectedFile(file);
                        toggleFileExpanded(file.path);
                      }}
                      class={`w-full p-3 flex items-center gap-3 hover:bg-secondary/50 transition-colors ${
                        selectedFile() === file ? "bg-secondary" : ""
                      }`}
                    >
                      <span class={`text-lg font-bold ${getStatusColor(file.status)}`}>
                        {getStatusIcon(file.status)}
                      </span>
                      
                      <div class="flex-1 text-left">
                        <p class="font-medium text-sm">{file.path}</p>
                        <div class="flex gap-3 text-xs text-muted-foreground mt-1">
                          <span class="text-green-500">+{file.additions}</span>
                          <span class="text-red-500">-{file.deletions}</span>
                        </div>
                      </div>
                      
                      <svg
                        class={`w-4 h-4 transition-transform ${
                          expandedFiles().has(file.path) ? "rotate-90" : ""
                        }`}
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                      </svg>
                    </button>
                    
                    <Show when={expandedFiles().has(file.path)}>
                      <div class="p-4 bg-[#0d0d0d] border-t border-border">
                        <pre class="text-xs font-mono overflow-x-auto whitespace-pre">
                          <code innerHTML={getDiff(file).split('\n').map(line => {
                            if (line.startsWith('+')) {
                              return `<span class="text-green-400 bg-green-500/10">${line}</span>`;
                            } else if (line.startsWith('-')) {
                              return `<span class="text-red-400 bg-red-500/10">${line}</span>`;
                            }
                            return `<span class="text-gray-400">${line}</span>`;
                          }).join('\n')} />
                        </pre>
                      </div>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </div>
          
          {/* Actions */}
          <div class="absolute bottom-0 left-0 right-0 bg-background border-t border-border p-4">
            <div class="flex gap-2">
              <button 
                onClick={() => {
                  showToast("Changes rejected", "warning");
                  props.onClose();
                }}
                class="btn btn-secondary flex-1"
              >
                Reject Changes
              </button>
              <button 
                onClick={() => {
                  showToast("All changes accepted!", "success");
                  showToast("Creating commit...", "info");
                  setTimeout(() => {
                    showToast("Changes committed successfully", "success");
                    props.onClose();
                  }, 1500);
                }}
                class="btn btn-primary flex-1"
              >
                Accept All
              </button>
            </div>
            <p class="text-xs text-center text-muted-foreground mt-2">
              Swipe left/right to navigate files
            </p>
          </div>
        </div>
      </div>
    </Portal>
  );
}