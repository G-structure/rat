import { createEffect, onCleanup, onMount } from "solid-js";
import { EditorView, basicSetup } from "codemirror";
import { EditorState, Extension } from "@codemirror/state";
import { keymap } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import { oneDark } from "@codemirror/theme-one-dark";

// Language imports
import { javascript } from "@codemirror/lang-javascript";
import { python } from "@codemirror/lang-python";
import { rust } from "@codemirror/lang-rust";
import { json } from "@codemirror/lang-json";

interface CodeMirrorProps {
  value: string;
  onChange?: (value: string) => void;
  language?: string;
  readOnly?: boolean;
  placeholder?: string;
  class?: string;
  onSelectionChange?: (selection: { from: number; to: number; text: string }) => void;
}

export function CodeMirror(props: CodeMirrorProps) {
  let editorRef: HTMLDivElement | undefined;
  let view: EditorView | undefined;
  
  const getLanguageExtension = (lang?: string): Extension[] => {
    switch (lang?.toLowerCase()) {
      case "javascript":
      case "js":
        return [javascript()];
      case "typescript":
      case "ts":
      case "tsx":
      case "jsx":
        return [javascript({ typescript: true, jsx: true })];
      case "python":
      case "py":
        return [python()];
      case "rust":
      case "rs":
        return [rust()];
      case "json":
        return [json()];
      default:
        return [];
    }
  };
  
  // Mobile-optimized theme extensions
  const mobileExtensions: Extension[] = [
    EditorView.theme({
      "&": {
        fontSize: "14px",
        height: "100%"
      },
      ".cm-scroller": {
        fontFamily: "JetBrains Mono, SF Mono, Consolas, monospace",
        padding: "12px 4px",
        overscrollBehavior: "contain"
      },
      ".cm-content": {
        padding: "0"
      },
      ".cm-line": {
        padding: "0 8px"
      },
      ".cm-gutters": {
        backgroundColor: "transparent",
        borderRight: "1px solid rgba(255, 255, 255, 0.1)"
      },
      ".cm-lineNumbers": {
        minWidth: "40px"
      },
      ".cm-cursor": {
        borderLeftWidth: "2px"
      },
      ".cm-selectionBackground": {
        backgroundColor: "rgba(59, 130, 246, 0.3)"
      },
      "&.cm-focused .cm-selectionBackground": {
        backgroundColor: "rgba(59, 130, 246, 0.4)"
      },
      ".cm-activeLine": {
        backgroundColor: "rgba(255, 255, 255, 0.05)"
      },
      ".cm-activeLineGutter": {
        backgroundColor: "rgba(255, 255, 255, 0.05)"
      },
      // Mobile-specific styles
      ".cm-content.cm-readOnly": {
        WebkitUserSelect: "text",
        userSelect: "text"
      },
      "@media (max-width: 768px)": {
        "&": {
          fontSize: "13px"
        },
        ".cm-lineNumbers": {
          minWidth: "32px"
        }
      }
    }),
    EditorView.lineWrapping,
    keymap.of([indentWithTab])
  ];
  
  onMount(() => {
    if (!editorRef) return;
    
    const updateListener = EditorView.updateListener.of((update) => {
      if (update.docChanged && props.onChange) {
        const value = update.state.doc.toString();
        props.onChange(value);
      }
      
      if (update.selectionSet && props.onSelectionChange) {
        const selection = update.state.selection.main;
        const text = update.state.doc.sliceString(selection.from, selection.to);
        props.onSelectionChange({
          from: selection.from,
          to: selection.to,
          text
        });
      }
    });
    
    const extensions = [
      basicSetup,
      oneDark,
      ...mobileExtensions,
      ...getLanguageExtension(props.language),
      updateListener,
      EditorState.readOnly.of(props.readOnly || false)
    ];
    
    if (props.placeholder) {
      extensions.push(EditorView.placeholder(props.placeholder));
    }
    
    const state = EditorState.create({
      doc: props.value,
      extensions
    });
    
    view = new EditorView({
      state,
      parent: editorRef
    });
    
    // Handle touch events for better mobile experience
    const handleTouchStart = (e: TouchEvent) => {
      // Prevent default only if we're not selecting text
      if (!window.getSelection()?.toString()) {
        e.stopPropagation();
      }
    };
    
    editorRef.addEventListener("touchstart", handleTouchStart, { passive: true });
    
    onCleanup(() => {
      editorRef?.removeEventListener("touchstart", handleTouchStart);
    });
  });
  
  // Update content when value prop changes
  createEffect(() => {
    if (view && props.value !== view.state.doc.toString()) {
      view.dispatch({
        changes: {
          from: 0,
          to: view.state.doc.length,
          insert: props.value
        }
      });
    }
  });
  
  // Update readonly state
  createEffect(() => {
    if (view) {
      view.dispatch({
        effects: EditorState.readOnly.reconfigure(EditorView.editable.of(!props.readOnly))
      });
    }
  });
  
  onCleanup(() => {
    view?.destroy();
  });
  
  return (
    <div 
      ref={editorRef}
      class={`h-full overflow-hidden ${props.class || ""}`}
    />
  );
}