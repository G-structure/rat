import { createSignal, For, Show } from "solid-js";
import { Portal } from "solid-js/web";

interface Branch {
  name: string;
  commit: {
    sha: string;
    url: string;
  };
  protected?: boolean;
}

interface BranchSelectorProps {
  branches: Branch[];
  currentBranch: string;
  onBranchChange: (branch: string) => void;
  isLoading?: boolean;
}

export function BranchSelector(props: BranchSelectorProps) {
  const [isOpen, setIsOpen] = createSignal(false);
  const [search, setSearch] = createSignal("");
  
  const filteredBranches = () => {
    const searchTerm = search().toLowerCase();
    return props.branches.filter(branch => 
      branch.name.toLowerCase().includes(searchTerm)
    );
  };
  
  const handleBranchSelect = (branch: string) => {
    props.onBranchChange(branch);
    setIsOpen(false);
    setSearch("");
  };
  
  return (
    <div class="relative">
      <button
        onClick={() => setIsOpen(!isOpen())}
        disabled={props.isLoading}
        class="px-3 py-1.5 text-sm bg-secondary rounded-lg flex items-center gap-2 hover:bg-secondary/80 transition-colors disabled:opacity-50"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
        </svg>
        <span class="font-medium">{props.currentBranch}</span>
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      
      <Show when={isOpen()}>
        <Portal>
          <div 
            class="fixed inset-0 z-40"
            onClick={() => setIsOpen(false)}
          />
          <div class="absolute top-full mt-2 left-0 w-72 max-h-96 bg-background border border-border rounded-xl shadow-xl z-50 overflow-hidden animate-fade-in">
            <div class="p-3 border-b border-border">
              <div class="relative">
                <input
                  type="text"
                  value={search()}
                  onInput={(e) => setSearch(e.currentTarget.value)}
                  placeholder="Search branches..."
                  class="w-full pl-9 pr-3 py-2 bg-secondary rounded-lg text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                  onClick={(e) => e.stopPropagation()}
                />
                <svg class="w-4 h-4 absolute left-3 top-2.5 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
              </div>
            </div>
            
            <div class="overflow-y-auto max-h-72">
              <Show when={filteredBranches().length === 0}>
                <div class="p-4 text-center text-sm text-muted-foreground">
                  No branches found
                </div>
              </Show>
              
              <div class="py-1">
                <For each={filteredBranches()}>
                  {(branch) => (
                    <button
                      onClick={() => handleBranchSelect(branch.name)}
                      class="w-full px-3 py-2 text-left hover:bg-secondary/50 transition-colors flex items-center justify-between group"
                    >
                      <div class="flex items-center gap-2 min-w-0">
                        <Show when={branch.name === props.currentBranch}>
                          <svg class="w-4 h-4 text-primary flex-shrink-0" fill="currentColor" viewBox="0 0 24 24">
                            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
                          </svg>
                        </Show>
                        <Show when={branch.name !== props.currentBranch}>
                          <div class="w-4 h-4 flex-shrink-0" />
                        </Show>
                        <span class="truncate font-medium">{branch.name}</span>
                      </div>
                      <Show when={branch.protected}>
                        <span class="text-xs px-2 py-0.5 bg-yellow-500/10 text-yellow-500 rounded-full">
                          Protected
                        </span>
                      </Show>
                    </button>
                  )}
                </For>
              </div>
            </div>
          </div>
        </Portal>
      </Show>
    </div>
  );
}