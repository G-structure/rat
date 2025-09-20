import { createSignal, For, Show } from "solid-js";
import { store } from "../state";

interface ParsedDiffLine {
  type: 'add' | 'remove' | 'context' | 'header';
  content: string;
  lineNumber?: number;
  isEditable?: boolean;
}

export function EditableDiffView() {
  const [editingLines, setEditingLines] = createSignal<Record<string, string>>({});
  const [hoveredLine, setHoveredLine] = createSignal<string | null>(null);
  
  const diffs = () => {
    const id = store.activeSessionId();
    return id ? (store.sessions()[id]?.diffs ?? []) : [];
  };
  
  const parseDiff = (diff: string): ParsedDiffLine[] => {
    const lines = diff.split('\n');
    const parsed: ParsedDiffLine[] = [];
    let addLineNumber = 0;
    
    lines.forEach((line) => {
      if (line.startsWith('+++') || line.startsWith('---') || line.startsWith('@@')) {
        parsed.push({ type: 'header', content: line });
      } else if (line.startsWith('+')) {
        addLineNumber++;
        parsed.push({ 
          type: 'add', 
          content: line.substring(1),
          lineNumber: addLineNumber,
          isEditable: true
        });
      } else if (line.startsWith('-')) {
        parsed.push({ type: 'remove', content: line.substring(1) });
      } else {
        parsed.push({ type: 'context', content: line });
      }
    });
    
    return parsed;
  };
  
  const handleLineEdit = (fileIndex: number, lineIndex: number, value: string) => {
    const key = `${fileIndex}-${lineIndex}`;
    setEditingLines(prev => ({ ...prev, [key]: value }));
  };
  
  const handleLineDoubleClick = (fileIndex: number, lineIndex: number) => {
    const key = `${fileIndex}-${lineIndex}`;
    const line = parseDiff(diffs()[fileIndex].diff)[lineIndex];
    if (line.type === 'add' && line.isEditable) {
      setEditingLines(prev => ({ ...prev, [key]: line.content }));
    }
  };
  
  const handleSaveLine = (fileIndex: number, lineIndex: number) => {
    const key = `${fileIndex}-${lineIndex}`;
    // Here you would typically save the changes back to the state or API
    console.log('Saving line:', editingLines()[key]);
    setEditingLines(prev => {
      const updated = { ...prev };
      delete updated[key];
      return updated;
    });
  };
  
  const isEditing = (fileIndex: number, lineIndex: number) => {
    const key = `${fileIndex}-${lineIndex}`;
    return key in editingLines();
  };
  
  return (
    <div class="diff-view" style="background:#0a0f10; padding: 16px;">
      <For each={diffs()}>
        {(d, fileIndex) => (
          <div style="margin-bottom: 24px; border: 1px solid #2d3748; border-radius: 8px; overflow: hidden;">
            <div style="background: #1a202c; padding: 12px 16px; border-bottom: 1px solid #2d3748;">
              <div style="color:#b6c2d6; font-weight:600; font-family: monospace;">
                {d.path}
              </div>
            </div>
            
            <div style="padding: 0;">
              <For each={parseDiff(d.diff)}>
                {(line, lineIndex) => {
                  const key = `${fileIndex()}-${lineIndex()}`;
                  const isLineEditing = () => isEditing(fileIndex(), lineIndex());
                  const editValue = () => editingLines()[key] || line.content;
                  
                  return (
                    <Show 
                      when={line.type !== 'remove'} 
                      fallback={
                        <div 
                          class="diff-line diff-remove"
                          style="
                            padding: 4px 16px;
                            font-family: 'JetBrains Mono', monospace;
                            font-size: 13px;
                            white-space: pre;
                            overflow-x: auto;
                            background: rgba(239, 68, 68, 0.1);
                            color: #ef4444;
                            opacity: 0.5;
                            text-decoration: line-through;
                          "
                        >
                          <span style="display: inline-block; width: 20px; text-align: center; opacity: 0.5;">-</span>
                          {line.content}
                        </div>
                      }
                    >
                      <div 
                        class={`diff-line diff-${line.type}`}
                        style={`
                          padding: 4px 16px;
                          font-family: 'JetBrains Mono', monospace;
                          font-size: 13px;
                          white-space: pre;
                          overflow-x: auto;
                          position: relative;
                          ${line.type === 'add' ? `
                            background: rgba(34, 197, 94, 0.1);
                            color: #22c55e;
                            cursor: ${isLineEditing() ? 'text' : 'pointer'};
                          ` : line.type === 'header' ? `
                            background: #1a202c;
                            color: #6b7280;
                            font-size: 12px;
                          ` : `
                            background: transparent;
                            color: #9ca3af;
                          `}
                        `}
                        onMouseEnter={() => line.type === 'add' && setHoveredLine(key)}
                        onMouseLeave={() => setHoveredLine(null)}
                        onDblClick={() => handleLineDoubleClick(fileIndex(), lineIndex())}
                      >
                        <Show
                          when={line.type === 'add' && isLineEditing()}
                          fallback={
                            <>
                              <span style="display: inline-block; width: 20px; text-align: center; opacity: 0.7;">
                                {line.type === 'add' ? '+' : line.type === 'context' ? ' ' : ''}
                              </span>
                              {line.content}
                              <Show when={line.type === 'add' && hoveredLine() === key && !isLineEditing()}>
                                <span 
                                  style="
                                    position: absolute; 
                                    right: 16px; 
                                    top: 50%; 
                                    transform: translateY(-50%);
                                    font-size: 11px;
                                    color: #6b7280;
                                    background: #1f2937;
                                    padding: 2px 8px;
                                    border-radius: 4px;
                                  "
                                >
                                  Double-click to edit
                                </span>
                              </Show>
                            </>
                          }
                        >
                          <span style="display: inline-block; width: 20px; text-align: center; opacity: 0.7;">+</span>
                          <input
                            type="text"
                            value={editValue()}
                            onInput={(e) => handleLineEdit(fileIndex(), lineIndex(), e.currentTarget.value)}
                            onKeyDown={(e) => {
                              if (e.key === 'Enter') {
                                handleSaveLine(fileIndex(), lineIndex());
                              } else if (e.key === 'Escape') {
                                setEditingLines(prev => {
                                  const updated = { ...prev };
                                  delete updated[key];
                                  return updated;
                                });
                              }
                            }}
                            onBlur={() => handleSaveLine(fileIndex(), lineIndex())}
                            style="
                              background: rgba(34, 197, 94, 0.2);
                              border: 1px solid #22c55e;
                              color: #22c55e;
                              font-family: inherit;
                              font-size: inherit;
                              padding: 2px 4px;
                              border-radius: 4px;
                              outline: none;
                              width: calc(100% - 30px);
                            "
                            autofocus
                          />
                        </Show>
                      </div>
                    </Show>
                  );
                }}
              </For>
            </div>
          </div>
        )}
      </For>
      
      <For each={diffs().length === 0 ? [1] : []}>
        {() => (
          <div style="color:#6b7a90; padding:8px;">
            No diffs available yet.
          </div>
        )}
      </For>
    </div>
  );
}