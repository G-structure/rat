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

// System prompt for code suggestions
const CODE_SUGGESTION_PROMPT = `You are an AI assistant that provides intelligent code suggestions and completions.

When given code context and a cursor position, you should:
1. Analyze the surrounding code and understand the patterns
2. Generate relevant suggestions based on context
3. Provide multiple completion options when appropriate
4. Include helpful documentation for complex suggestions
5. Consider the programming language and common idioms

Return suggestions in JSON format:
{
  "suggestions": [
    {
      "text": "The code to insert",
      "label": "Short label for the suggestion",
      "detail": "Optional description",
      "documentation": "Optional markdown documentation",
      "insertText": "Text to insert with placeholders like \${1:param}",
      "type": "function|variable|snippet|keyword"
    }
  ]
}`;

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
    const { context, position, trigger } = body;
    
    if (!context?.fileContent) {
      return new Response(
        JSON.stringify({ error: "File content is required" }),
        {
          status: 400,
          headers: { "Content-Type": "application/json" }
        }
      );
    }
    
    // Extract relevant context around cursor position
    const lines = context.fileContent.split("\n");
    const currentLine = position?.line || 0;
    const currentColumn = position?.column || 0;
    
    // Get context window (5 lines before and after)
    const startLine = Math.max(0, currentLine - 5);
    const endLine = Math.min(lines.length - 1, currentLine + 5);
    const contextWindow = lines.slice(startLine, endLine + 1).join("\n");
    
    // Current line content
    const lineContent = lines[currentLine] || "";
    const beforeCursor = lineContent.substring(0, currentColumn);
    const afterCursor = lineContent.substring(currentColumn);
    
    // Prepare the AI request
    const aiRequest = {
      messages: [
        {
          role: "system",
          content: CODE_SUGGESTION_PROMPT
        },
        {
          role: "user",
          content: `Language: ${context.language || "unknown"}
File: ${context.fileName || "untitled"}
Trigger: ${trigger || "manual"}

Context around cursor:
\`\`\`${context.language || ""}
${contextWindow}
\`\`\`

Current line: "${lineContent}"
Before cursor: "${beforeCursor}"
After cursor: "${afterCursor}"

Provide code completion suggestions for this position.`
        }
      ],
      max_tokens: 1024,
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
    
    // Parse the response
    const content = result.content?.[0]?.text || "{}";
    let suggestions;
    
    try {
      // Try to parse as JSON first
      const jsonMatch = content.match(/\{[\s\S]*\}/);
      if (jsonMatch) {
        suggestions = JSON.parse(jsonMatch[0]);
      } else {
        // Fallback: create a single suggestion from the response
        suggestions = {
          suggestions: [{
            text: content.trim(),
            label: "AI suggestion",
            type: "snippet"
          }]
        };
      }
    } catch (e) {
      suggestions = {
        suggestions: [{
          text: content.trim(),
          label: "AI suggestion",
          type: "snippet"
        }]
      };
    }
    
    return new Response(
      JSON.stringify(suggestions),
      {
        headers: { "Content-Type": "application/json" }
      }
    );
  } catch (error) {
    console.error("Suggestion generation error:", error);
    return new Response(
      JSON.stringify({ error: "Failed to generate suggestions" }),
      {
        status: 500,
        headers: { "Content-Type": "application/json" }
      }
    );
  }
};