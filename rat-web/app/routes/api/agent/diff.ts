import type { APIEvent } from "solid-start/api";

// Helper to get session
async function getSession(event: APIEvent) {
  const cookieHeader = event.request.headers.get("Cookie");
  const sid = cookieHeader?.match(/sid=([^;]+)/)?.[1];
  
  if (!sid) return null;
  
  const { SESSIONS } = event.env as any;
  if (!SESSIONS) return null;
  
  return await SESSIONS.get(`session:${sid}`, "json");
}

// System prompt for surgical edits
const SURGICAL_EDIT_PROMPT = `You are an AI assistant that generates precise code edits in unified diff format.

When given a code file and a user prompt, you must:
1. Analyze the code and understand the requested changes
2. Generate a unified diff patch that can be applied with standard patch tools
3. Make minimal, surgical changes that accomplish the user's goal
4. Preserve all formatting, indentation, and style conventions
5. Return ONLY the unified diff format with no additional explanation

Example output format:
\`\`\`diff
--- a/src/components/Button.tsx
+++ b/src/components/Button.tsx
@@ -5,7 +5,7 @@
 export function Button({ children, onClick }) {
   return (
-    <button class="btn" onClick={onClick}>
+    <button class="btn btn-primary" onClick={onClick}>
       {children}
     </button>
   )
\`\`\`

Rules:
- Use proper unified diff format with @@ line markers
- Include 3 lines of context before and after changes
- Mark removed lines with - prefix
- Mark added lines with + prefix
- Preserve exact whitespace and indentation
- Only modify what's necessary for the requested change`;

export const onRequestPost = async (event: APIEvent) => {
  const session = await getSession(event);
  
  if (!session) {
    return new Response(JSON.stringify({ error: "Unauthorized" }), {
      status: 401,
      headers: { "Content-Type": "application/json" }
    });
  }
  
  const { AI_AGENT_URL, AI_AGENT_KEY } = event.env as any;
  
  if (!AI_AGENT_URL || !AI_AGENT_KEY) {
    return new Response(
      JSON.stringify({ error: "AI service not configured" }),
      {
        status: 503,
        headers: { "Content-Type": "application/json" }
      }
    );
  }
  
  try {
    const body = await event.request.json();
    const { prompt, context } = body;
    
    if (!prompt || !context?.fileContent) {
      return new Response(
        JSON.stringify({ error: "Prompt and file content are required" }),
        {
          status: 400,
          headers: { "Content-Type": "application/json" }
        }
      );
    }
    
    // Prepare the AI request
    const aiRequest = {
      messages: [
        {
          role: "system",
          content: SURGICAL_EDIT_PROMPT
        },
        {
          role: "user",
          content: `File: ${context.fileName || "untitled"}
Language: ${context.language || "unknown"}

Current code:
\`\`\`${context.language || ""}
${context.fileContent}
\`\`\`

User request: ${prompt}

Generate a unified diff patch for the requested changes.`
        }
      ],
      max_tokens: 2048,
      temperature: 0.3,
      model: "claude-3-opus-20240229"
    };
    
    // Call AI agent
    const response = await fetch(AI_AGENT_URL, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Authorization": `Bearer ${AI_AGENT_KEY}`,
        "Anthropic-Version": "2023-06-01"
      },
      body: JSON.stringify(aiRequest)
    });
    
    if (!response.ok) {
      throw new Error(`AI service error: ${response.status}`);
    }
    
    const result = await response.json();
    
    // Extract the diff from the response
    const content = result.content?.[0]?.text || "";
    const diffMatch = content.match(/```diff\n([\s\S]*?)```/);
    const diff = diffMatch ? diffMatch[1].trim() : content;
    
    // Parse the diff to extract hunks
    const hunks = parseDiffToHunks(diff);
    
    return new Response(
      JSON.stringify({
        format: "unified-diff@1",
        diff,
        hunks,
        files: [{
          path: context.fileName || "untitled",
          patch: diff
        }]
      }),
      {
        headers: { "Content-Type": "application/json" }
      }
    );
  } catch (error) {
    console.error("Diff generation error:", error);
    return new Response(
      JSON.stringify({ error: "Failed to generate diff" }),
      {
        status: 500,
        headers: { "Content-Type": "application/json" }
      }
    );
  }
};

// Parse unified diff into hunks for the UI
function parseDiffToHunks(diff: string) {
  const hunks: any[] = [];
  const lines = diff.split("\n");
  
  let currentHunk: any = null;
  let lineNumber = 0;
  
  for (const line of lines) {
    if (line.startsWith("@@")) {
      // Parse hunk header: @@ -start,count +start,count @@
      const match = line.match(/@@ -(\d+),?\d* \+(\d+),?\d* @@/);
      if (match) {
        if (currentHunk) {
          hunks.push(currentHunk);
        }
        currentHunk = {
          oldStart: parseInt(match[1]),
          newStart: parseInt(match[2]),
          lines: []
        };
        lineNumber = currentHunk.newStart;
      }
    } else if (currentHunk) {
      if (line.startsWith("+")) {
        currentHunk.lines.push({
          type: "add",
          content: line.substring(1),
          lineNumber: lineNumber++
        });
      } else if (line.startsWith("-")) {
        currentHunk.lines.push({
          type: "remove",
          content: line.substring(1),
          lineNumber: -1
        });
      } else {
        currentHunk.lines.push({
          type: "context",
          content: line,
          lineNumber: lineNumber++
        });
      }
    }
  }
  
  if (currentHunk) {
    hunks.push(currentHunk);
  }
  
  return hunks;
}