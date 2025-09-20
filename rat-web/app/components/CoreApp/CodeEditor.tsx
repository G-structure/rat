import { createSignal, onMount, createEffect } from "solid-js";

interface CodeEditorProps {
  file: string | null;
  onTextSelect?: (text: string) => void;
}

// Mock code content for different files
const getFileContent = (filePath: string): string => {
  // React Todo App files
  if (filePath === "src/components/TodoList.tsx") {
    return `import { For } from 'solid-js';
import { TodoItem } from './TodoItem';
import { Todo } from '../types/todo';

interface TodoListProps {
  todos: Todo[];
  onToggle: (id: string) => void;
  onDelete: (id: string) => void;
}

export function TodoList(props: TodoListProps) {
  return (
    <div class="space-y-2">
      <For each={props.todos}>
        {(todo) => (
          <TodoItem
            todo={todo}
            onToggle={() => props.onToggle(todo.id)}
            onDelete={() => props.onDelete(todo.id)}
          />
        )}
      </For>
    </div>
  );
}`;
  } else if (filePath === "src/types/todo.ts") {
    return `export interface Todo {
  id: string;
  text: string;
  completed: boolean;
  createdAt: Date;
}

export type TodoFilter = 'all' | 'active' | 'completed';`;
  }
  
  // Python API files
  else if (filePath === "app/main.py") {
    return `from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from app.api import users, products, auth
from app.services.database import init_db

app = FastAPI(title="E-Commerce API", version="1.0.0")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)

@app.on_event("startup")
async def startup_event():
    await init_db()

@app.get("/")
async def root():
    return {"message": "Welcome to E-Commerce API"}

app.include_router(auth.router, prefix="/api/auth", tags=["auth"])
app.include_router(users.router, prefix="/api/users", tags=["users"])
app.include_router(products.router, prefix="/api/products", tags=["products"])`;
  } else if (filePath === "app/models/user.py") {
    return `from sqlalchemy import Column, String, DateTime, Boolean
from sqlalchemy.ext.declarative import declarative_base
from datetime import datetime
import uuid

Base = declarative_base()

class User(Base):
    __tablename__ = "users"
    
    id = Column(String, primary_key=True, default=lambda: str(uuid.uuid4()))
    email = Column(String, unique=True, nullable=False)
    username = Column(String, unique=True, nullable=False)
    hashed_password = Column(String, nullable=False)
    is_active = Column(Boolean, default=True)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)`;
  }
  
  // Swift files
  else if (filePath === "ShoppingApp/Views/ProductListView.swift") {
    return `import SwiftUI

struct ProductListView: View {
    @StateObject private var viewModel = ProductListViewModel()
    @State private var searchText = ""
    
    var body: some View {
        NavigationView {
            List {
                ForEach(viewModel.filteredProducts) { product in
                    NavigationLink(destination: ProductDetailView(product: product)) {
                        ProductRow(product: product)
                    }
                }
            }
            .searchable(text: $searchText)
            .navigationTitle("Products")
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: { viewModel.loadProducts() }) {
                        Image(systemName: "arrow.clockwise")
                    }
                }
            }
        }
        .onAppear {
            viewModel.loadProducts()
        }
    }
}`;
  } else if (filePath === "ShoppingApp/Models/Product.swift") {
    return `import Foundation

struct Product: Identifiable, Codable {
    let id: UUID
    let name: String
    let description: String
    let price: Double
    let imageURL: String
    let category: String
    let stock: Int
    var isFavorite: Bool = false
    
    var formattedPrice: String {
        let formatter = NumberFormatter()
        formatter.numberStyle = .currency
        return formatter.string(from: NSNumber(value: price)) ?? "$0.00"
    }
}`;
  }
  
  // ML Python files
  else if (filePath === "src/models/collaborative_filtering.py") {
    return `import numpy as np
import pandas as pd
from sklearn.metrics.pairwise import cosine_similarity
from scipy.sparse import csr_matrix

class CollaborativeFiltering:
    def __init__(self, n_factors=50, learning_rate=0.01, n_epochs=20):
        self.n_factors = n_factors
        self.learning_rate = learning_rate
        self.n_epochs = n_epochs
        self.user_factors = None
        self.item_factors = None
        
    def fit(self, ratings_matrix):
        """Train the collaborative filtering model using matrix factorization."""
        self.ratings_matrix = csr_matrix(ratings_matrix)
        n_users, n_items = ratings_matrix.shape
        
        # Initialize factors
        self.user_factors = np.random.normal(0, 0.1, (n_users, self.n_factors))
        self.item_factors = np.random.normal(0, 0.1, (n_items, self.n_factors))
        
        # Training loop
        for epoch in range(self.n_epochs):
            self._sgd_step()
            if epoch % 5 == 0:
                loss = self._compute_loss()
                print(f"Epoch {epoch}, Loss: {loss:.4f}")
    
    def predict(self, user_id, item_id):
        """Predict rating for a user-item pair."""
        return np.dot(self.user_factors[user_id], self.item_factors[item_id])
    
    def recommend(self, user_id, n_recommendations=10):
        """Get top N recommendations for a user."""
        user_vector = self.user_factors[user_id]
        scores = np.dot(self.item_factors, user_vector)
        top_items = np.argsort(scores)[::-1][:n_recommendations]
        return top_items`;
  }
  
  // Vue Dashboard files
  else if (filePath === "src/components/ChartWidget.vue") {
    return `<template>
  <div class="chart-widget">
    <div class="widget-header">
      <h3>{{ title }}</h3>
      <select v-model="timeRange" @change="updateChart">
        <option value="7d">Last 7 days</option>
        <option value="30d">Last 30 days</option>
        <option value="90d">Last 90 days</option>
      </select>
    </div>
    <div class="chart-container" ref="chartContainer"></div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch } from 'vue'
import * as d3 from 'd3'

const props = defineProps({
  title: String,
  data: Array,
  type: {
    type: String,
    default: 'line'
  }
})

const timeRange = ref('7d')
const chartContainer = ref(null)

const updateChart = () => {
  // D3.js chart implementation
  const svg = d3.select(chartContainer.value)
  // ... chart drawing logic
}

onMounted(() => {
  updateChart()
})

watch(() => props.data, updateChart)
</script>

<style scoped>
.chart-widget {
  background: white;
  border-radius: 8px;
  padding: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.widget-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.chart-container {
  height: 300px;
}
</style>`;
  }
  
  // Return existing mock content or generate default
  return mockFileContents[filePath] || `// File: ${filePath}\n// Content not available in mock mode\n\nconsole.log('Hello from ${filePath}');`;
};

const mockFileContents: Record<string, string> = {
  "src/App.tsx": `import React from 'react';
import { Header } from './components/Header';
import { Button } from './components/Button';

function App() {
  const [count, setCount] = React.useState(0);
  
  const handleClick = () => {
    setCount(count + 1);
  };
  
  return (
    <div className="app">
      <Header title="My App" />
      <div className="content">
        <h1>Welcome to React</h1>
        <p>Count: {count}</p>
        <Button onClick={handleClick}>
          Click me
        </Button>
      </div>
    </div>
  );
}

export default App;`,
  
  "src/components/Button.tsx": `import React from 'react';

interface ButtonProps {
  children: React.ReactNode;
  onClick?: () => void;
  variant?: 'primary' | 'secondary';
  disabled?: boolean;
}

export function Button({ 
  children, 
  onClick, 
  variant = 'primary',
  disabled = false 
}: ButtonProps) {
  return (
    <button
      className={\`btn btn-\${variant}\`}
      onClick={onClick}
      disabled={disabled}
    >
      {children}
    </button>
  );
}`,
  
  "src/utils/helpers.ts": `export function formatDate(date: Date): string {
  return new Intl.DateTimeFormat('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  }).format(date);
}

export function debounce<T extends (...args: any[]) => void>(
  func: T,
  wait: number
): T {
  let timeout: NodeJS.Timeout;
  
  return ((...args: Parameters<T>) => {
    clearTimeout(timeout);
    timeout = setTimeout(() => func(...args), wait);
  }) as T;
}`,
  
  "package.json": `{
  "name": "my-awesome-app",
  "version": "1.0.0",
  "description": "A sample React application",
  "main": "index.js",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  },
  "devDependencies": {
    "vite": "^4.5.0",
    "@vitejs/plugin-react": "^4.0.0"
  }
}`
};

export function CodeEditor(props: CodeEditorProps) {
  const [content, setContent] = createSignal("");
  const [selectedText, setSelectedText] = createSignal("");
  let editorRef: HTMLDivElement | undefined;
  
  // Update content when file changes
  createEffect(() => {
    if (props.file) {
      setContent(getFileContent(props.file));
    }
  });
  
  const handleTextSelection = () => {
    const selection = window.getSelection();
    if (selection && selection.toString().trim()) {
      const text = selection.toString();
      setSelectedText(text);
      props.onTextSelect?.(text);
    }
  };
  
  onMount(() => {
    if (editorRef) {
      editorRef.addEventListener("mouseup", handleTextSelection);
      editorRef.addEventListener("touchend", handleTextSelection);
    }
  });
  
  const getLineNumbers = () => {
    const lines = content().split("\n");
    return lines.map((_, i) => i + 1).join("\n");
  };
  
  const highlightCode = (code: string) => {
    // Simple syntax highlighting
    return code
      .replace(/\b(import|export|function|const|let|var|class|return|if|else|for|while)\b/g, '<span class="text-purple-400">$1</span>')
      .replace(/\b(React|useState|useEffect)\b/g, '<span class="text-blue-400">$1</span>')
      .replace(/(["'])([^"']*)\1/g, '<span class="text-green-400">$1$2$1</span>')
      .replace(/\/\/.*$/gm, '<span class="text-gray-500">$&</span>')
      .replace(/\b(\d+)\b/g, '<span class="text-orange-400">$1</span>');
  };
  
  return (
    <div class="h-full flex bg-[#0d0d0d]">
      {/* Line Numbers */}
      <div class="w-12 bg-[#0d0d0d] border-r border-border text-muted-foreground text-sm py-4 px-2 text-right select-none">
        <pre class="font-mono leading-6">{getLineNumbers()}</pre>
      </div>
      
      {/* Editor Content */}
      <div class="flex-1 overflow-auto">
        <div
          ref={editorRef}
          class="min-h-full p-4 font-mono text-sm leading-6 outline-none"
          contentEditable
          spellcheck={false}
          innerHTML={highlightCode(content())}
          onInput={(e) => setContent(e.currentTarget.textContent || "")}
          style={{
            "caret-color": "#fff",
            "white-space": "pre-wrap",
            "word-break": "break-word"
          }}
        />
      </div>
      
      {/* Selection Indicator */}
      {selectedText() && (
        <div class="fixed bottom-20 right-4 bg-primary text-primary-foreground px-3 py-2 rounded-lg text-sm shadow-lg animate-fade-in">
          Text selected: {selectedText().substring(0, 30)}...
        </div>
      )}
    </div>
  );
}