import { createSignal, createRoot } from "solid-js";

interface FileContent {
  path: string;
  content: string;
  originalContent: string;
  isDirty: boolean;
}

interface EditorAction {
  type: "edit" | "create" | "delete" | "move";
  timestamp: number;
  filePath: string;
  previousState?: string;
  newState?: string;
  metadata?: any;
}

interface EditorState {
  files: Map<string, FileContent>;
  history: EditorAction[];
  historyIndex: number;
  activeFile: string | null;
}

function createEditorStore() {
  const [state, setState] = createSignal<EditorState>({
    files: new Map(),
    history: [],
    historyIndex: -1,
    activeFile: null
  });

  // Get or create file content
  const getFileContent = (path: string): string => {
    const fileContent = state().files.get(path);
    return fileContent?.content || "";
  };

  // Set file content
  const setFileContent = (path: string, content: string, recordHistory = true) => {
    setState(prev => {
      const files = new Map(prev.files);
      const existing = files.get(path);
      
      if (recordHistory) {
        // Add to history
        const newHistory = prev.history.slice(0, prev.historyIndex + 1);
        newHistory.push({
          type: "edit",
          timestamp: Date.now(),
          filePath: path,
          previousState: existing?.content || "",
          newState: content
        });
        
        files.set(path, {
          path,
          content,
          originalContent: existing?.originalContent || content,
          isDirty: content !== (existing?.originalContent || content)
        });
        
        return {
          ...prev,
          files,
          history: newHistory,
          historyIndex: newHistory.length - 1
        };
      } else {
        // Update without recording history (for undo/redo)
        files.set(path, {
          path,
          content,
          originalContent: existing?.originalContent || content,
          isDirty: content !== (existing?.originalContent || content)
        });
        
        return {
          ...prev,
          files
        };
      }
    });
  };

  // Create new file
  const createFile = (path: string, initialContent: string = "") => {
    setState(prev => {
      const files = new Map(prev.files);
      
      // Add to history
      const newHistory = prev.history.slice(0, prev.historyIndex + 1);
      newHistory.push({
        type: "create",
        timestamp: Date.now(),
        filePath: path,
        newState: initialContent
      });
      
      files.set(path, {
        path,
        content: initialContent,
        originalContent: initialContent,
        isDirty: false
      });
      
      return {
        ...prev,
        files,
        history: newHistory,
        historyIndex: newHistory.length - 1,
        activeFile: path
      };
    });
  };

  // Move file
  const moveFile = (oldPath: string, newPath: string) => {
    setState(prev => {
      const files = new Map(prev.files);
      const fileContent = files.get(oldPath);
      
      if (fileContent) {
        // Add to history
        const newHistory = prev.history.slice(0, prev.historyIndex + 1);
        newHistory.push({
          type: "move",
          timestamp: Date.now(),
          filePath: oldPath,
          metadata: { newPath }
        });
        
        files.delete(oldPath);
        files.set(newPath, {
          ...fileContent,
          path: newPath
        });
        
        return {
          ...prev,
          files,
          history: newHistory,
          historyIndex: newHistory.length - 1,
          activeFile: prev.activeFile === oldPath ? newPath : prev.activeFile
        };
      }
      
      return prev;
    });
  };

  // Undo action
  const undo = () => {
    setState(prev => {
      if (prev.historyIndex < 0) return prev;
      
      const action = prev.history[prev.historyIndex];
      const files = new Map(prev.files);
      
      switch (action.type) {
        case "edit":
          if (action.previousState !== undefined) {
            const file = files.get(action.filePath);
            if (file) {
              files.set(action.filePath, {
                ...file,
                content: action.previousState,
                isDirty: action.previousState !== file.originalContent
              });
            }
          }
          break;
          
        case "create":
          files.delete(action.filePath);
          break;
          
        case "move":
          if (action.metadata?.newPath) {
            const file = files.get(action.metadata.newPath);
            if (file) {
              files.delete(action.metadata.newPath);
              files.set(action.filePath, {
                ...file,
                path: action.filePath
              });
            }
          }
          break;
      }
      
      return {
        ...prev,
        files,
        historyIndex: prev.historyIndex - 1
      };
    });
  };

  // Redo action
  const redo = () => {
    setState(prev => {
      if (prev.historyIndex >= prev.history.length - 1) return prev;
      
      const action = prev.history[prev.historyIndex + 1];
      const files = new Map(prev.files);
      
      switch (action.type) {
        case "edit":
          if (action.newState !== undefined) {
            const file = files.get(action.filePath);
            if (file) {
              files.set(action.filePath, {
                ...file,
                content: action.newState,
                isDirty: action.newState !== file.originalContent
              });
            }
          }
          break;
          
        case "create":
          files.set(action.filePath, {
            path: action.filePath,
            content: action.newState || "",
            originalContent: action.newState || "",
            isDirty: false
          });
          break;
          
        case "move":
          if (action.metadata?.newPath) {
            const file = files.get(action.filePath);
            if (file) {
              files.delete(action.filePath);
              files.set(action.metadata.newPath, {
                ...file,
                path: action.metadata.newPath
              });
            }
          }
          break;
      }
      
      return {
        ...prev,
        files,
        historyIndex: prev.historyIndex + 1
      };
    });
  };

  // Check if can undo/redo
  const canUndo = () => state().historyIndex >= 0;
  const canRedo = () => state().historyIndex < state().history.length - 1;

  // Set active file
  const setActiveFile = (path: string | null) => {
    setState(prev => ({ ...prev, activeFile: path }));
  };

  // Get file template based on extension
  const getFileTemplate = (filename: string): string => {
    const ext = filename.split('.').pop()?.toLowerCase();
    
    switch (ext) {
      case 'tsx':
        return `import { Component } from 'solid-js';

interface ${filename.replace(/\.(tsx?|jsx?)$/, '')}Props {
  // Add props here
}

export const ${filename.replace(/\.(tsx?|jsx?)$/, '')}: Component<${filename.replace(/\.(tsx?|jsx?)$/, '')}Props> = (props) => {
  return (
    <div>
      <h1>${filename.replace(/\.(tsx?|jsx?)$/, '')}</h1>
    </div>
  );
};
`;
      
      case 'ts':
        return `// ${filename}

export function example() {
  console.log('Hello from ${filename}');
}
`;
      
      case 'css':
        return `/* ${filename} */

.container {
  /* Add styles here */
}
`;
      
      case 'json':
        return `{
  "name": "${filename.replace('.json', '')}",
  "version": "1.0.0"
}
`;
      
      default:
        return `// ${filename}\n\n`;
    }
  };

  return {
    state,
    getFileContent,
    setFileContent,
    createFile,
    moveFile,
    undo,
    redo,
    canUndo,
    canRedo,
    setActiveFile,
    getFileTemplate
  };
}

export const editorStore = createRoot(createEditorStore);