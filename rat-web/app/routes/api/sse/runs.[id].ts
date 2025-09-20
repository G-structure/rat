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

export const onRequestGet = async (event: APIEvent & { params: { id: string } }) => {
  const session = await getSession(event);
  
  if (!session) {
    return new Response("Unauthorized", { status: 401 });
  }
  
  const runId = event.params.id;
  
  // Set up SSE headers
  const headers = new Headers({
    "Content-Type": "text/event-stream",
    "Cache-Control": "no-cache",
    "Connection": "keep-alive",
    "Access-Control-Allow-Origin": "*"
  });
  
  // Create a TransformStream for SSE
  const { readable, writable } = new TransformStream();
  const writer = writable.getWriter();
  
  // Send initial connection message
  writer.write(new TextEncoder().encode(`event: connected\ndata: {"runId":"${runId}"}\n\n`));
  
  // Simulate streaming AI responses (in production, this would connect to your AI service)
  const sendUpdate = async () => {
    try {
      // Mock progress updates
      const updates = [
        { event: "start", data: { message: "Analyzing code..." } },
        { event: "progress", data: { percent: 20, message: "Understanding context..." } },
        { event: "progress", data: { percent: 50, message: "Generating changes..." } },
        { event: "progress", data: { percent: 80, message: "Validating edits..." } },
        { event: "result", data: { 
          diff: "--- a/file.js\n+++ b/file.js\n@@ -1,3 +1,3 @@\n-console.log('hello');\n+console.log('Hello, World!');\n",
          summary: "Capitalized and added punctuation to console message"
        }},
        { event: "complete", data: { success: true } }
      ];
      
      for (const update of updates) {
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        const message = `event: ${update.event}\ndata: ${JSON.stringify(update.data)}\n\n`;
        await writer.write(new TextEncoder().encode(message));
        
        if (update.event === "complete") {
          break;
        }
      }
    } catch (error) {
      const errorMessage = `event: error\ndata: ${JSON.stringify({ error: "Stream failed" })}\n\n`;
      await writer.write(new TextEncoder().encode(errorMessage));
    } finally {
      await writer.close();
    }
  };
  
  // Start sending updates
  sendUpdate();
  
  return new Response(readable, { headers });
};

// Create a new run
export const onRequestPost = async (event: APIEvent) => {
  const session = await getSession(event);
  
  if (!session) {
    return new Response(JSON.stringify({ error: "Unauthorized" }), {
      status: 401,
      headers: { "Content-Type": "application/json" }
    });
  }
  
  try {
    const body = await event.request.json();
    const { prompt, context, type } = body;
    
    if (!prompt) {
      return new Response(
        JSON.stringify({ error: "Prompt is required" }),
        {
          status: 400,
          headers: { "Content-Type": "application/json" }
        }
      );
    }
    
    // Generate a unique run ID
    const runId = crypto.randomUUID();
    
    // In production, you would:
    // 1. Create the run in your database
    // 2. Queue the AI task
    // 3. Return the run ID for SSE subscription
    
    const { RUNS } = event.env as any;
    if (RUNS) {
      // Store run metadata in KV
      await RUNS.put(
        `run:${runId}`,
        JSON.stringify({
          id: runId,
          userId: session.user.id,
          prompt,
          context,
          type: type || "diff",
          status: "pending",
          created: Date.now()
        }),
        { expirationTtl: 60 * 60 * 24 } // 24 hours
      );
    }
    
    return new Response(
      JSON.stringify({
        runId,
        status: "pending",
        streamUrl: `/api/sse/runs/${runId}`
      }),
      {
        headers: { "Content-Type": "application/json" }
      }
    );
  } catch (error) {
    console.error("Run creation error:", error);
    return new Response(
      JSON.stringify({ error: "Failed to create run" }),
      {
        status: 500,
        headers: { "Content-Type": "application/json" }
      }
    );
  }
};