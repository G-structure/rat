import { For, Show, createSignal } from "solid-js";
import { useNavigate } from "@solidjs/router";

interface FileTreeItem {
  name: string;
  path: string;
  type: "file" | "dir";
  size?: number;
  sha?: string;
}

interface FileTreeProps {
  items: FileTreeItem[];
  basePath: string;
  onFileSelect?: (file: FileTreeItem) => void;
}

export function FileTree(props: FileTreeProps) {
  const navigate = useNavigate();
  
  const getFileIcon = (fileName: string, type: "file" | "dir") => {
    if (type === "dir") {
      return (
        <svg class="w-5 h-5 text-blue-500" fill="currentColor" viewBox="0 0 24 24">
          <path d="M10 4H4c-1.11 0-2 .89-2 2v12c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V8c0-1.11-.89-2-2-2h-8l-2-2z"/>
        </svg>
      );
    }
    
    // File type detection based on extension
    const ext = fileName.split(".").pop()?.toLowerCase();
    
    if (["js", "jsx", "ts", "tsx"].includes(ext || "")) {
      return (
        <svg class="w-5 h-5 text-yellow-500" fill="currentColor" viewBox="0 0 24 24">
          <path d="M12,10.11C13.03,10.11 13.87,10.95 13.87,12C13.87,13 13.03,13.85 12,13.85C10.97,13.85 10.13,13 10.13,12C10.13,10.95 10.97,10.11 12,10.11M7.37,20C8,20.38 9.38,19.8 10.97,18.3C10.45,17.71 9.94,17.07 9.46,16.4C8.64,16.32 7.83,16.2 7.06,16.04C6.55,18.18 6.74,19.65 7.37,20M8.08,14.03L7.79,13.28C7.68,13.04 7.57,12.8 7.47,12.56C7.06,12.67 6.67,12.79 6.28,12.93C6.4,13.31 6.53,13.69 6.68,14.07L8.08,14.03M13.88,11.24C13.82,11.03 13.76,10.82 13.69,10.61C13.96,10.54 14.24,10.47 14.53,10.42C14.43,10.73 14.33,11.05 14.22,11.37L13.88,11.24M20.89,9.29C20.76,8.61 20.37,8.06 19.8,7.74C19.23,7.42 18.5,7.37 17.89,7.59L17.95,8.05L18.03,8.5C18,8.61 17.96,8.72 17.91,8.82C17.86,8.94 17.79,9.05 17.71,9.14C17.62,9.23 17.5,9.29 17.39,9.32C16.72,9.5 16,8.96 15.81,8.28C15.38,6.58 13.86,5.53 12.16,5.94L11.71,6.06C10.53,6.37 9.61,7.27 9.27,8.36C9.3,8.5 9.35,8.61 9.4,8.71C9.46,8.82 9.54,8.92 9.63,9L9.67,9.03C10.26,9.43 11.07,9.28 11.47,8.7C11.71,8.35 12.11,8.15 12.54,8.14H12.58C13.22,8.13 13.78,8.55 13.97,9.16C14.57,10.83 16.53,11.74 18.2,11.15L18.65,11L18.67,11C19.26,10.81 19.73,10.38 19.97,9.82C20.26,10.39 20.53,11.07 20.72,11.87C21.5,15 20.36,17.31 18.04,18.12C17.4,17.5 16.68,16.87 15.89,16.25C15.95,15.5 15.95,14.73 15.89,13.91C16.77,13.87 17.63,13.77 18.5,13.65C18.22,12.93 17.89,12.22 17.5,11.56C16.86,11.72 16.22,11.85 15.56,11.96C15.29,11.3 14.97,10.67 14.63,10.07C14.43,9.72 14.21,9.37 14,9.03C11.75,5.72 8.85,3.82 6.5,4.72C4.14,5.61 3.39,8.8 4.62,12.37C4.95,13.27 5.35,14.15 5.82,15L5.83,15L5.85,15C6.04,15.37 6.25,15.73 6.47,16.08C7.27,17.19 8.2,18.19 9.22,19.04C9.82,19.62 10.46,20.14 11.11,20.58C13.46,22.07 15.85,22.37 17.5,21.5C20,20.08 20.91,16.65 20.34,12.58M11.35,19.5C10.72,19.15 10.11,18.71 9.54,18.21C10.12,18.24 10.69,18.24 11.27,18.23C11.3,18.64 11.33,19.06 11.35,19.5V19.5M12,16.97C11.38,16.97 10.77,16.95 10.17,16.91C10,16.79 9.88,16.67 9.73,16.54C9.44,16.26 9.16,15.96 8.9,15.63C8.76,15.44 8.63,15.24 8.5,15.03C9.19,15.14 9.89,15.23 10.6,15.28C10.73,15.96 10.89,16.62 11.09,17.26C11.1,17.29 11.12,17.33 11.14,17.37C11.43,17.41 11.71,17.44 12,17.45C12.29,17.44 12.58,17.41 12.86,17.37L12.94,17.17C13.12,16.53 13.28,15.87 13.4,15.19C14.11,15.14 14.81,15.05 15.5,14.94C15.37,15.14 15.24,15.35 15.1,15.54C14.84,15.87 14.56,16.17 14.27,16.45C14.13,16.57 14,16.69 13.85,16.8C13.24,16.88 12.62,16.93 12,16.97M10.69,13.75C10.66,13.5 10.64,13.26 10.62,13C10.64,12.76 10.66,12.53 10.69,12.3C11.13,12.33 11.56,12.34 12,12.34C12.45,12.34 12.88,12.33 13.32,12.3C13.35,12.53 13.37,12.77 13.39,13.03L13.4,13.23C13.38,13.5 13.36,13.73 13.32,13.95C12.88,13.93 12.44,13.91 12,13.91C11.57,13.91 11.13,13.93 10.69,13.95V13.75M12.21,11.12C12.14,11.12 12.07,11.12 12,11.12C11.97,11.12 11.94,11.12 11.91,11.12C11.85,10.92 11.79,10.72 11.74,10.5C11.84,10.26 11.95,10 12.05,9.76L12.15,9.55C12.25,9.74 12.35,9.94 12.44,10.14C12.54,10.36 12.64,10.59 12.72,10.82C12.56,10.96 12.38,11.05 12.21,11.12V11.12Z"/>
        </svg>
      );
    }
    
    if (["py", "pyx", "pyw"].includes(ext || "")) {
      return (
        <svg class="w-5 h-5 text-blue-400" fill="currentColor" viewBox="0 0 24 24">
          <path d="M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2M12,4C16.41,4 20,7.59 20,12C20,16.41 16.41,20 12,20C7.59,20 4,16.41 4,12C4,7.59 7.59,4 12,4M9.5,7.5A1,1 0 0,0 8.5,8.5A1,1 0 0,0 9.5,9.5A1,1 0 0,0 10.5,8.5A1,1 0 0,0 9.5,7.5M14.5,7.5A1,1 0 0,0 13.5,8.5A1,1 0 0,0 14.5,9.5A1,1 0 0,0 15.5,8.5A1,1 0 0,0 14.5,7.5M12,10.5L7,15.5V17H17V15.5L12,10.5Z"/>
        </svg>
      );
    }
    
    if (["json", "jsonc"].includes(ext || "")) {
      return (
        <svg class="w-5 h-5 text-amber-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4"/>
        </svg>
      );
    }
    
    if (["md", "mdx"].includes(ext || "")) {
      return (
        <svg class="w-5 h-5 text-gray-500" fill="currentColor" viewBox="0 0 24 24">
          <path d="M5,3C3.89,3 3,3.89 3,5V19C3,20.11 3.89,21 5,21H19C20.11,21 21,20.11 21,19V5C21,3.89 20.11,3 19,3H5M5,5H19V19H5V5M7,7V17H9V11.5L11,17H13L15,11.5V17H17V7H14L12,12L10,7H7Z"/>
        </svg>
      );
    }
    
    // Default file icon
    return (
      <svg class="w-5 h-5 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
      </svg>
    );
  };
  
  const formatSize = (size?: number) => {
    if (!size) return "";
    const kb = size / 1024;
    if (kb < 1024) return `${kb.toFixed(1)} KB`;
    const mb = kb / 1024;
    return `${mb.toFixed(1)} MB`;
  };
  
  const handleItemClick = (item: FileTreeItem) => {
    if (item.type === "dir") {
      navigate(`${props.basePath}/${item.name}`);
    } else {
      if (props.onFileSelect) {
        props.onFileSelect(item);
      } else {
        navigate(`${props.basePath}/${item.name}`);
      }
    }
  };
  
  return (
    <div class="space-y-1">
      <For each={props.items}>
        {(item) => (
          <button
            onClick={() => handleItemClick(item)}
            class="w-full p-3 rounded-xl hover:bg-secondary/50 transition-colors text-left flex items-center gap-3 group"
          >
            {getFileIcon(item.name, item.type)}
            <div class="flex-1 min-w-0">
              <p class="font-medium truncate group-hover:text-primary transition-colors">
                {item.name}
              </p>
              <Show when={item.type === "file" && item.size !== undefined}>
                <p class="text-xs text-muted-foreground">
                  {formatSize(item.size)}
                </p>
              </Show>
            </div>
            <svg class="w-5 h-5 text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </button>
        )}
      </For>
    </div>
  );
}