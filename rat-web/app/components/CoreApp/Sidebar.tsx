import { createSignal, For, Show, createMemo, onMount, onCleanup } from "solid-js";
import { editorStore } from "~/stores/editorStore";

interface SidebarProps {
  open: boolean;
  onToggle: () => void;
  selectedFile: string | null;
  onFileSelect: (file: string) => void;
  repoName?: string;
}

interface FileNode {
  name: string;
  path: string;
  type: "file" | "folder";
  children?: FileNode[];
  expanded?: boolean;
}

export function Sidebar(props: SidebarProps) {
  const [showNewFileDialog, setShowNewFileDialog] = createSignal(false);
  const [newFileName, setNewFileName] = createSignal("");
  const [newFilePath, setNewFilePath] = createSignal("");
  const [draggedNode, setDraggedNode] = createSignal<FileNode | null>(null);
  const [dragOverNode, setDragOverNode] = createSignal<string | null>(null);
  
  // Generate different file structures based on repo
  const getRepoStructure = (): FileNode[] => {
    const repoName = props.repoName || "default-project";
    
    if (repoName === "react-todo-app") {
      return [
        {
          name: "src",
          type: "folder",
          path: "src",
          expanded: true,
          children: [
            { name: "App.tsx", type: "file", path: "src/App.tsx" },
            { name: "main.tsx", type: "file", path: "src/main.tsx" },
            { name: "index.css", type: "file", path: "src/index.css" },
            {
              name: "components",
              type: "folder",
              path: "src/components",
              expanded: true,
              children: [
                { name: "TodoList.tsx", type: "file", path: "src/components/TodoList.tsx" },
                { name: "TodoItem.tsx", type: "file", path: "src/components/TodoItem.tsx" },
                { name: "AddTodo.tsx", type: "file", path: "src/components/AddTodo.tsx" },
                { name: "TodoFilter.tsx", type: "file", path: "src/components/TodoFilter.tsx" }
              ]
            },
            {
              name: "hooks",
              type: "folder",
              path: "src/hooks",
              children: [
                { name: "useTodos.ts", type: "file", path: "src/hooks/useTodos.ts" },
                { name: "useLocalStorage.ts", type: "file", path: "src/hooks/useLocalStorage.ts" }
              ]
            },
            {
              name: "types",
              type: "folder",
              path: "src/types",
              children: [
                { name: "todo.ts", type: "file", path: "src/types/todo.ts" }
              ]
            }
          ]
        },
        { name: "package.json", type: "file", path: "package.json" },
        { name: "tsconfig.json", type: "file", path: "tsconfig.json" },
        { name: "vite.config.ts", type: "file", path: "vite.config.ts" },
        { name: "README.md", type: "file", path: "README.md" }
      ];
    } else if (repoName === "python-api-server") {
      return [
        {
          name: "app",
          type: "folder",
          path: "app",
          expanded: true,
          children: [
            { name: "__init__.py", type: "file", path: "app/__init__.py" },
            { name: "main.py", type: "file", path: "app/main.py" },
            { name: "config.py", type: "file", path: "app/config.py" },
            {
              name: "api",
              type: "folder",
              path: "app/api",
              expanded: true,
              children: [
                { name: "__init__.py", type: "file", path: "app/api/__init__.py" },
                { name: "users.py", type: "file", path: "app/api/users.py" },
                { name: "products.py", type: "file", path: "app/api/products.py" },
                { name: "auth.py", type: "file", path: "app/api/auth.py" }
              ]
            },
            {
              name: "models",
              type: "folder",
              path: "app/models",
              children: [
                { name: "__init__.py", type: "file", path: "app/models/__init__.py" },
                { name: "user.py", type: "file", path: "app/models/user.py" },
                { name: "product.py", type: "file", path: "app/models/product.py" }
              ]
            },
            {
              name: "services",
              type: "folder",
              path: "app/services",
              children: [
                { name: "__init__.py", type: "file", path: "app/services/__init__.py" },
                { name: "database.py", type: "file", path: "app/services/database.py" },
                { name: "email.py", type: "file", path: "app/services/email.py" }
              ]
            }
          ]
        },
        { name: "requirements.txt", type: "file", path: "requirements.txt" },
        { name: "Dockerfile", type: "file", path: "Dockerfile" },
        { name: "docker-compose.yml", type: "file", path: "docker-compose.yml" },
        { name: "pytest.ini", type: "file", path: "pytest.ini" },
        { name: "README.md", type: "file", path: "README.md" }
      ];
    } else if (repoName === "mobile-shopping-app") {
      return [
        {
          name: "ShoppingApp",
          type: "folder",
          path: "ShoppingApp",
          expanded: true,
          children: [
            { name: "AppDelegate.swift", type: "file", path: "ShoppingApp/AppDelegate.swift" },
            { name: "ContentView.swift", type: "file", path: "ShoppingApp/ContentView.swift" },
            {
              name: "Views",
              type: "folder",
              path: "ShoppingApp/Views",
              expanded: true,
              children: [
                { name: "ProductListView.swift", type: "file", path: "ShoppingApp/Views/ProductListView.swift" },
                { name: "ProductDetailView.swift", type: "file", path: "ShoppingApp/Views/ProductDetailView.swift" },
                { name: "CartView.swift", type: "file", path: "ShoppingApp/Views/CartView.swift" },
                { name: "ProfileView.swift", type: "file", path: "ShoppingApp/Views/ProfileView.swift" }
              ]
            },
            {
              name: "Models",
              type: "folder",
              path: "ShoppingApp/Models",
              children: [
                { name: "Product.swift", type: "file", path: "ShoppingApp/Models/Product.swift" },
                { name: "CartItem.swift", type: "file", path: "ShoppingApp/Models/CartItem.swift" },
                { name: "User.swift", type: "file", path: "ShoppingApp/Models/User.swift" }
              ]
            },
            {
              name: "Services",
              type: "folder",
              path: "ShoppingApp/Services",
              children: [
                { name: "APIService.swift", type: "file", path: "ShoppingApp/Services/APIService.swift" },
                { name: "CartManager.swift", type: "file", path: "ShoppingApp/Services/CartManager.swift" }
              ]
            }
          ]
        },
        { name: "Package.swift", type: "file", path: "Package.swift" },
        { name: "Info.plist", type: "file", path: "Info.plist" }
      ];
    } else if (repoName === "ml-recommendation-engine") {
      return [
        {
          name: "src",
          type: "folder",
          path: "src",
          expanded: true,
          children: [
            { name: "__init__.py", type: "file", path: "src/__init__.py" },
            { name: "train.py", type: "file", path: "src/train.py" },
            { name: "predict.py", type: "file", path: "src/predict.py" },
            {
              name: "models",
              type: "folder",
              path: "src/models",
              expanded: true,
              children: [
                { name: "collaborative_filtering.py", type: "file", path: "src/models/collaborative_filtering.py" },
                { name: "content_based.py", type: "file", path: "src/models/content_based.py" },
                { name: "hybrid.py", type: "file", path: "src/models/hybrid.py" }
              ]
            },
            {
              name: "data",
              type: "folder",
              path: "src/data",
              children: [
                { name: "preprocessing.py", type: "file", path: "src/data/preprocessing.py" },
                { name: "loaders.py", type: "file", path: "src/data/loaders.py" }
              ]
            },
            {
              name: "utils",
              type: "folder",
              path: "src/utils",
              children: [
                { name: "metrics.py", type: "file", path: "src/utils/metrics.py" },
                { name: "visualization.py", type: "file", path: "src/utils/visualization.py" }
              ]
            }
          ]
        },
        {
          name: "notebooks",
          type: "folder",
          path: "notebooks",
          children: [
            { name: "exploration.ipynb", type: "file", path: "notebooks/exploration.ipynb" },
            { name: "model_evaluation.ipynb", type: "file", path: "notebooks/model_evaluation.ipynb" }
          ]
        },
        { name: "requirements.txt", type: "file", path: "requirements.txt" },
        { name: "setup.py", type: "file", path: "setup.py" },
        { name: "README.md", type: "file", path: "README.md" }
      ];
    } else if (repoName === "vue-dashboard") {
      return [
        {
          name: "src",
          type: "folder",
          path: "src",
          expanded: true,
          children: [
            { name: "App.vue", type: "file", path: "src/App.vue" },
            { name: "main.js", type: "file", path: "src/main.js" },
            {
              name: "components",
              type: "folder",
              path: "src/components",
              expanded: true,
              children: [
                { name: "DashboardLayout.vue", type: "file", path: "src/components/DashboardLayout.vue" },
                { name: "ChartWidget.vue", type: "file", path: "src/components/ChartWidget.vue" },
                { name: "MetricCard.vue", type: "file", path: "src/components/MetricCard.vue" },
                { name: "DataTable.vue", type: "file", path: "src/components/DataTable.vue" }
              ]
            },
            {
              name: "views",
              type: "folder",
              path: "src/views",
              children: [
                { name: "Overview.vue", type: "file", path: "src/views/Overview.vue" },
                { name: "Analytics.vue", type: "file", path: "src/views/Analytics.vue" },
                { name: "Reports.vue", type: "file", path: "src/views/Reports.vue" }
              ]
            },
            {
              name: "store",
              type: "folder",
              path: "src/store",
              children: [
                { name: "index.js", type: "file", path: "src/store/index.js" },
                { name: "modules/", type: "folder", path: "src/store/modules" }
              ]
            }
          ]
        },
        { name: "package.json", type: "file", path: "package.json" },
        { name: "vite.config.js", type: "file", path: "vite.config.js" },
        { name: "tailwind.config.js", type: "file", path: "tailwind.config.js" }
      ];
    }
    
    // Default structure
    return [
      {
        name: "src",
        type: "folder",
        path: "src",
        expanded: true,
        children: [
          { name: "App.tsx", type: "file", path: "src/App.tsx" },
          { name: "index.tsx", type: "file", path: "src/index.tsx" },
          { name: "index.css", type: "file", path: "src/index.css" },
          {
            name: "components",
            type: "folder",
            path: "src/components",
            expanded: false,
            children: [
              { name: "Header.tsx", type: "file", path: "src/components/Header.tsx" },
              { name: "Sidebar.tsx", type: "file", path: "src/components/Sidebar.tsx" },
              { name: "Footer.tsx", type: "file", path: "src/components/Footer.tsx" }
            ]
          },
          {
            name: "utils",
            type: "folder",
            path: "src/utils",
            expanded: false,
            children: [
              { name: "api.ts", type: "file", path: "src/utils/api.ts" },
              { name: "helpers.ts", type: "file", path: "src/utils/helpers.ts" }
            ]
          }
        ]
      },
      { name: "package.json", type: "file", path: "package.json" },
      { name: "tsconfig.json", type: "file", path: "tsconfig.json" },
      { name: "README.md", type: "file", path: "README.md" }
    ];
  };
  
  // Initialize file tree based on repo
  const [fileTree, setFileTree] = createSignal<FileNode[]>(getRepoStructure());
  
  // Update file tree when repo changes
  createMemo(() => {
    setFileTree(getRepoStructure());
  });
  
  // Handle new file creation
  const handleNewFile = () => {
    const fileName = newFileName();
    const filePath = newFilePath() || "src";
    
    if (!fileName) return;
    
    const fullPath = `${filePath}/${fileName}`;
    
    // Update file tree with new file
    const updateTree = (nodes: FileNode[]): FileNode[] => {
      return nodes.map(node => {
        if (node.path === filePath && node.type === "folder") {
          const newFile: FileNode = {
            name: fileName,
            path: fullPath,
            type: "file"
          };
          return {
            ...node,
            children: [...(node.children || []), newFile],
            expanded: true
          };
        }
        if (node.children) {
          return { ...node, children: updateTree(node.children) };
        }
        return node;
      });
    };
    
    // Create file in editor store with template content
    const template = editorStore.getFileTemplate(fileName);
    editorStore.createFile(fullPath, template);
    
    setFileTree(updateTree(fileTree()));
    props.onFileSelect(fullPath);
    setShowNewFileDialog(false);
    setNewFileName("");
    setNewFilePath("");
  };
  
  // Handle drag and drop
  const moveNode = (sourcePath: string, targetPath: string) => {
    let movedNode: FileNode | null = null;
    
    // Extract the node to move
    const extractNode = (nodes: FileNode[]): FileNode[] => {
      return nodes.filter(node => {
        if (node.path === sourcePath) {
          movedNode = node;
          return false;
        }
        if (node.children) {
          node.children = extractNode(node.children);
        }
        return true;
      });
    };
    
    // Insert the node at target location
    const insertNode = (nodes: FileNode[]): FileNode[] => {
      return nodes.map(node => {
        if (node.path === targetPath && node.type === "folder" && movedNode) {
          return {
            ...node,
            children: [...(node.children || []), movedNode],
            expanded: true
          };
        }
        if (node.children) {
          return { ...node, children: insertNode(node.children) };
        }
        return node;
      });
    };
    
    const treeWithoutSource = extractNode([...fileTree()]);
    if (movedNode) {
      setFileTree(insertNode(treeWithoutSource));
      // Update the path in editor store
      const newPath = `${targetPath}/${movedNode.name}`;
      editorStore.moveFile(sourcePath, newPath);
    }
  };

  const toggleFolder = (path: string) => {
    setFileTree(tree => {
      const toggleNode = (nodes: FileNode[]): FileNode[] => {
        return nodes.map(node => {
          if (node.path === path && node.type === "folder") {
            return { ...node, expanded: !node.expanded };
          }
          if (node.children) {
            return { ...node, children: toggleNode(node.children) };
          }
          return node;
        });
      };
      return toggleNode(tree);
    });
  };

  const FileTreeItem = (props: { node: FileNode; level: number }) => {
    const [isDragging, setIsDragging] = createSignal(false);
    const isSelected = () => props.node.path === props.selectedFile;
    
    const handleClick = () => {
      if (props.node.type === "folder") {
        toggleFolder(props.node.path);
      } else {
        props.onFileSelect(props.node.path);
      }
    };
    
    return (
      <div 
        draggable={true}
        onDragStart={(e) => {
          setIsDragging(true);
          setDraggedNode(props.node);
          e.dataTransfer!.effectAllowed = "move";
        }}
        onDragEnd={() => {
          setIsDragging(false);
          setDraggedNode(null);
          setDragOverNode(null);
        }}
        onDragOver={(e) => {
          e.preventDefault();
          if (props.node.type === "folder" && draggedNode() && draggedNode()!.path !== props.node.path) {
            setDragOverNode(props.node.path);
          }
        }}
        onDragLeave={() => {
          setDragOverNode(null);
        }}
        onDrop={(e) => {
          e.preventDefault();
          const dragged = draggedNode();
          if (dragged && props.node.type === "folder" && dragged.path !== props.node.path) {
            moveNode(dragged.path, props.node.path);
          }
          setDragOverNode(null);
        }}
      >
        <button
          onClick={handleClick}
          class={`w-full flex items-center gap-2 px-2 py-1.5 text-sm hover:bg-secondary/50 rounded transition-colors ${
            isSelected() ? "bg-secondary text-primary" : ""
          } ${
            isDragging() ? "opacity-50" : ""
          } ${
            dragOverNode() === props.node.path ? "bg-primary/20" : ""
          }`}
          style={{ "padding-left": `${props.level * 12 + 8}px` }}
        >
          <Show when={props.node.type === "folder"}>
            <svg
              class={`w-4 h-4 transition-transform ${props.node.expanded ? "rotate-90" : ""}`}
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </Show>
          
          <Show when={props.node.type === "folder"}>
            <svg class="w-4 h-4 text-blue-500" fill="currentColor" viewBox="0 0 24 24">
              <path d="M10 4H4c-1.11 0-2 .89-2 2v12c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V8c0-1.11-.89-2-2-2h-8l-2-2z"/>
            </svg>
          </Show>
          
          <Show when={props.node.type === "file"}>
            {props.node.name.endsWith('.tsx') || props.node.name.endsWith('.ts') ? (
              <svg class="w-4 h-4 text-blue-400" fill="currentColor" viewBox="0 0 24 24">
                <path d="M13,9H18.5L13,3.5V9M6,2H14L20,8V20A2,2 0 0,1 18,22H6C4.89,22 4,21.1 4,20V4C4,2.89 4.89,2 6,2M6.92,17.5H9.08V10.5L11,10.75V9H5V10.75L6.92,10.5V17.5Z"/>
              </svg>
            ) : props.node.name.endsWith('.css') ? (
              <svg class="w-4 h-4 text-pink-400" fill="currentColor" viewBox="0 0 24 24">
                <path d="M5,3L4.35,6.34H17.94L17.5,8.5H3.92L3.26,11.83H16.85L16.09,15.64L10.61,17.45L5.86,15.64L6.19,14H2.85L2.06,18L9.91,21L18.96,18L20.16,11.97L20.4,10.76L21.94,3H5Z"/>
              </svg>
            ) : props.node.name.endsWith('.py') ? (
              <svg class="w-4 h-4 text-yellow-400" fill="currentColor" viewBox="0 0 24 24">
                <path d="M19.14,7.5A2.86,2.86 0 0,1 22,10.36V14.14A2.86,2.86 0 0,1 19.14,17H12C12,17.39 12.32,17.96 12.71,17.96H17V19.64A2.86,2.86 0 0,1 14.14,22.5H9.86A2.86,2.86 0 0,1 7,19.64V15.89C7,14.31 8.28,13.04 9.86,13.04H18.46C19.62,13.04 20.09,12.28 20.09,11.93V10.36C20.09,9.66 19.66,9.21 18.93,9.21H9.86A2.86,2.86 0 0,1 7,6.36V3.5C7,2.56 7.56,2 8.5,2H13.5C14.44,2 15,2.56 15,3.5V5H16.5C17.44,5 18,5.56 18,6.5V7.5H19.14M12,4A1,1 0 0,0 11,5A1,1 0 0,0 12,6A1,1 0 0,0 13,5A1,1 0 0,0 12,4Z"/>
              </svg>
            ) : props.node.name.endsWith('.swift') ? (
              <svg class="w-4 h-4 text-orange-500" fill="currentColor" viewBox="0 0 24 24">
                <path d="M18.69,12.7C18.66,12.72 18.64,12.74 18.62,12.77C16.95,14.5 14,15.16 11.58,14.38L11.5,14.34C10.45,13.85 9.5,13.16 8.65,12.32C9.04,12.64 9.43,12.96 9.85,13.26L9.91,13.3C7.04,11.53 4.64,8.59 3.67,4.44C3.67,4.44 5.9,9.09 10.84,12.28L10.77,12.23C8.96,10.84 7.4,9.17 6.14,7.3C6.14,7.3 7.84,9.63 10.13,11.36L10.13,11.36C12.1,12.79 13.73,13.41 14.81,13.5L14.94,13.5C15.38,13.5 15.77,13.42 16.09,13.29C14.38,11.59 11.25,8.3 11.25,8.3C16.04,12.19 18.17,12.95 18.86,12.68C19.5,12.38 19.57,11.13 18.69,9.38C17.38,6.93 12.94,2.82 12.94,2.82S15.16,5.55 16.5,7.73C16.06,7.18 14.82,5.66 13.14,3.92C13.14,3.92 16.5,7.1 18,9.24L18.03,9.27C18.41,9.84 18.69,10.38 18.86,10.86C19,11.05 19.08,11.27 19.13,11.5C19.29,12.04 19.21,12.5 18.86,12.68V12.67H18.85L18.81,12.69C18.77,12.7 18.73,12.7 18.69,12.7M20,12A8,8 0 0,1 12,20A8,8 0 0,1 4,12A8,8 0 0,1 12,4A8,8 0 0,1 20,12Z"/>
              </svg>
            ) : props.node.name.endsWith('.vue') ? (
              <svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 24 24">
                <path d="M2,3H5.5L12,15L18.5,3H22L12,21L2,3M6.5,3H9.5L12,7.58L14.5,3H17.5L12,13.08L6.5,3Z"/>
              </svg>
            ) : props.node.name.endsWith('.json') ? (
              <svg class="w-4 h-4 text-yellow-600" fill="currentColor" viewBox="0 0 24 24">
                <path d="M5,3H7A2,2 0 0,1 9,5V17A2,2 0 0,1 7,19H5A2,2 0 0,1 3,17V5A2,2 0 0,1 5,3M14,3H19A2,2 0 0,1 21,5V7A2,2 0 0,1 19,9H16V11H19A2,2 0 0,1 21,13V17A2,2 0 0,1 19,19H14A2,2 0 0,1 12,17V13H14V17H19V13H14A2,2 0 0,1 12,11V9A2,2 0 0,1 14,7H19V5H14V9H12V5A2,2 0 0,1 14,3Z"/>
              </svg>
            ) : props.node.name.endsWith('.md') ? (
              <svg class="w-4 h-4 text-gray-400" fill="currentColor" viewBox="0 0 24 24">
                <path d="M20.56 18H3.44C2.65 18 2 17.37 2 16.59V7.41C2 6.63 2.65 6 3.44 6H20.56C21.35 6 22 6.63 22 7.41V16.59C22 17.37 21.35 18 20.56 18M6.81 15.19V11.53L8.73 13.88L10.65 11.53V15.19H12.58V8.81H10.65L8.73 11.16L6.81 8.81H4.89V15.19H6.81M19.11 12H17.19V8.81H15.26V12H13.34L16.23 15.28L19.11 12Z"/>
              </svg>
            ) : props.node.name.endsWith('.ipynb') ? (
              <svg class="w-4 h-4 text-orange-400" fill="currentColor" viewBox="0 0 24 24">
                <path d="M9.4,16.6L4.8,12L9.4,7.4L8,6L2,12L8,18L9.4,16.6M14.6,16.6L19.2,12L14.6,7.4L16,6L22,12L16,18L14.6,16.6Z"/>
              </svg>
            ) : (
              <svg class="w-4 h-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
            )}
          </Show>
          
          <span class="truncate">{props.node.name}</span>
        </button>
        
        <Show when={props.node.type === "folder" && props.node.expanded && props.node.children}>
          <For each={props.node.children}>
            {child => <FileTreeItem node={child} level={props.level + 1} />}
          </For>
        </Show>
      </div>
    );
  };

  return (
    <div
      class={`${
        props.open ? "w-64" : "w-0"
      } transition-all duration-300 border-r border-border bg-background flex flex-col overflow-hidden`}
    >
      <div class="h-12 border-b border-border flex items-center justify-between px-4 flex-shrink-0">
        <span class="font-semibold text-sm">Explorer</span>
        <button
          onClick={props.onToggle}
          class="p-1.5 hover:bg-secondary rounded-lg"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
          </svg>
        </button>
      </div>
      
      <div class="flex-1 overflow-y-auto p-2">
        <For each={fileTree()}>
          {node => <FileTreeItem node={node} level={0} />}
        </For>
      </div>
      
      <div class="p-4 border-t border-border">
        <button 
          class="w-full btn btn-secondary text-sm"
          onClick={() => setShowNewFileDialog(true)}
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          New File
        </button>
      </div>
      
      {/* New File Dialog */}
      <Show when={showNewFileDialog()}>
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div class="bg-background border border-border rounded-lg p-6 w-96">
            <h3 class="text-lg font-semibold mb-4">Create New File</h3>
            
            <div class="space-y-4">
              <div>
                <label class="block text-sm font-medium mb-2">File Name</label>
                <input
                  type="text"
                  value={newFileName()}
                  onInput={(e) => setNewFileName(e.currentTarget.value)}
                  placeholder="example.tsx"
                  class="w-full px-3 py-2 bg-secondary border border-border rounded-lg focus:outline-none focus:ring-2 focus:ring-primary"
                  autofocus
                />
              </div>
              
              <div>
                <label class="block text-sm font-medium mb-2">Folder Path (optional)</label>
                <input
                  type="text"
                  value={newFilePath()}
                  onInput={(e) => setNewFilePath(e.currentTarget.value)}
                  placeholder="src/components"
                  class="w-full px-3 py-2 bg-secondary border border-border rounded-lg focus:outline-none focus:ring-2 focus:ring-primary"
                />
              </div>
            </div>
            
            <div class="flex justify-end gap-2 mt-6">
              <button
                class="px-4 py-2 text-sm hover:bg-secondary rounded-lg"
                onClick={() => {
                  setShowNewFileDialog(false);
                  setNewFileName("");
                  setNewFilePath("");
                }}
              >
                Cancel
              </button>
              <button
                class="px-4 py-2 text-sm bg-primary hover:bg-primary/90 text-primary-foreground rounded-lg"
                onClick={handleNewFile}
                disabled={!newFileName()}
              >
                Create File
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}