import { createSignal, Show } from "solid-js";

interface MobileToolbarProps {
  onUndo?: () => void;
  onRedo?: () => void;
  onFormat?: () => void;
  onSearch?: () => void;
  onSave?: () => void;
  hasChanges?: boolean;
  readOnly?: boolean;
}

export function MobileToolbar(props: MobileToolbarProps) {
  const [showMore, setShowMore] = createSignal(false);
  
  return (
    <div class="border-t border-border bg-background/95 backdrop-blur-ios">
      <div class="flex items-center justify-between px-2 py-1">
        {/* Left side - Edit actions */}
        <div class="flex items-center gap-1">
          <Show when={!props.readOnly}>
            <button
              onClick={props.onUndo}
              class="p-2 rounded-lg hover:bg-secondary active:scale-95 transition-all"
              title="Undo"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6" />
              </svg>
            </button>
            
            <button
              onClick={props.onRedo}
              class="p-2 rounded-lg hover:bg-secondary active:scale-95 transition-all"
              title="Redo"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 10h-10a8 8 0 00-8 8v2m18-10l-6 6m6-6l-6-6" />
              </svg>
            </button>
            
            <div class="w-px h-6 bg-border mx-1" />
          </Show>
          
          <button
            onClick={props.onSearch}
            class="p-2 rounded-lg hover:bg-secondary active:scale-95 transition-all"
            title="Search"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </button>
          
          <Show when={!props.readOnly}>
            <button
              onClick={props.onFormat}
              class="p-2 rounded-lg hover:bg-secondary active:scale-95 transition-all"
              title="Format"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h8m-8 6h16" />
              </svg>
            </button>
          </Show>
        </div>
        
        {/* Right side - Save and more */}
        <div class="flex items-center gap-1">
          <Show when={!props.readOnly && props.onSave}>
            <button
              onClick={props.onSave}
              class={`px-3 py-1.5 rounded-lg font-medium transition-all ${
                props.hasChanges 
                  ? "bg-primary text-primary-foreground" 
                  : "bg-secondary text-muted-foreground"
              }`}
              disabled={!props.hasChanges}
            >
              Save
            </button>
          </Show>
          
          <button
            onClick={() => setShowMore(!showMore())}
            class="p-2 rounded-lg hover:bg-secondary active:scale-95 transition-all"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z" />
            </svg>
          </button>
        </div>
      </div>
      
      {/* Extended toolbar */}
      <Show when={showMore()}>
        <div class="border-t border-border px-2 py-2 flex flex-wrap gap-2">
          <button class="px-3 py-1.5 text-sm bg-secondary rounded-lg hover:bg-secondary/80">
            Go to line
          </button>
          <button class="px-3 py-1.5 text-sm bg-secondary rounded-lg hover:bg-secondary/80">
            Find & Replace
          </button>
          <button class="px-3 py-1.5 text-sm bg-secondary rounded-lg hover:bg-secondary/80">
            Toggle wrap
          </button>
          <button class="px-3 py-1.5 text-sm bg-secondary rounded-lg hover:bg-secondary/80">
            Settings
          </button>
        </div>
      </Show>
    </div>
  );
}

// Quick insert toolbar for common code snippets
export function QuickInsertBar(props: { onInsert: (text: string) => void; language?: string }) {
  const getSnippets = () => {
    switch (props.language?.toLowerCase()) {
      case "javascript":
      case "typescript":
        return [
          { label: "=>", text: " => " },
          { label: "{ }", text: "{ }" },
          { label: "[ ]", text: "[ ]" },
          { label: "( )", text: "( )" },
          { label: "``", text: "``" },
          { label: "//", text: "// " },
          { label: "log", text: "console.log()" },
          { label: "if", text: "if () {\n  \n}" }
        ];
      case "python":
        return [
          { label: ":", text: ":" },
          { label: "[ ]", text: "[]" },
          { label: "{ }", text: "{}" },
          { label: "( )", text: "()" },
          { label: '""', text: '""' },
          { label: "#", text: "# " },
          { label: "def", text: "def ():\n    " },
          { label: "if", text: "if :\n    " }
        ];
      default:
        return [
          { label: "{ }", text: "{ }" },
          { label: "[ ]", text: "[ ]" },
          { label: "( )", text: "( )" },
          { label: '""', text: '""' },
          { label: "//", text: "// " },
          { label: "/*", text: "/* */" },
          { label: "->", text: " -> " },
          { label: "=>", text: " => " }
        ];
    }
  };
  
  return (
    <div class="flex gap-1 px-2 py-1 overflow-x-auto scrollbar-none bg-secondary/50">
      {getSnippets().map(snippet => (
        <button
          onClick={() => props.onInsert(snippet.text)}
          class="px-2 py-1 text-xs font-mono bg-background rounded whitespace-nowrap hover:bg-primary hover:text-primary-foreground transition-colors"
        >
          {snippet.label}
        </button>
      ))}
    </div>
  );
}