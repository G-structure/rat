import { APIEvent } from "@solidjs/start/server";

export async function POST(event: APIEvent) {
  try {
    const body = await event.request.json();
    const { device_code } = body;
    
    if (!device_code) {
      return new Response(
        JSON.stringify({ error: "device_code is required" }),
        {
          status: 400,
          headers: { "Content-Type": "application/json" }
        }
      );
    }
    
    const platform = event.locals.runtime.platform;
    if (!platform?.env?.AUTH_SESSIONS) {
      // Mock success for development
      return new Response(
        JSON.stringify({
          access_token: `claude_token_${Math.random().toString(36).substr(2, 9)}`,
          token_type: "Bearer",
          scope: "read write",
          expires_in: 3600
        }),
        {
          status: 200,
          headers: { "Content-Type": "application/json" }
        }
      );
    }
    
    // Check pending auth status
    const pendingAuth = await platform.env.AUTH_SESSIONS.get(
      `claude_pending_${device_code}`
    );
    
    if (!pendingAuth) {
      return new Response(
        JSON.stringify({ error: "Invalid or expired device code" }),
        {
          status: 400,
          headers: { "Content-Type": "application/json" }
        }
      );
    }
    
    const authData = JSON.parse(pendingAuth);
    
    // Check if expired
    if (Date.now() > authData.expires_at) {
      await platform.env.AUTH_SESSIONS.delete(`claude_pending_${device_code}`);
      return new Response(
        JSON.stringify({ error: "Device code expired" }),
        {
          status: 400,
          headers: { "Content-Type": "application/json" }
        }
      );
    }
    
    // In a real implementation, this would check with Claude's API
    // For now, we'll simulate authorization after a few polls
    if (authData.poll_count && authData.poll_count > 2) {
      // Mark as authorized
      await platform.env.AUTH_SESSIONS.delete(`claude_pending_${device_code}`);
      
      const token = `claude_token_${Math.random().toString(36).substr(2, 9)}`;
      
      // Store the token for the user
      const sessionId = event.locals.sessionId;
      if (sessionId) {
        const sessionData = await platform.env.AUTH_SESSIONS.get(sessionId);
        if (sessionData) {
          const session = JSON.parse(sessionData);
          session.claude_token = token;
          await platform.env.AUTH_SESSIONS.put(
            sessionId,
            JSON.stringify(session),
            { expirationTtl: 86400 * 30 } // 30 days
          );
        }
      }
      
      return new Response(
        JSON.stringify({
          access_token: token,
          token_type: "Bearer",
          scope: "read write",
          expires_in: 3600
        }),
        {
          status: 200,
          headers: { "Content-Type": "application/json" }
        }
      );
    } else {
      // Still pending, increment poll count
      authData.poll_count = (authData.poll_count || 0) + 1;
      await platform.env.AUTH_SESSIONS.put(
        `claude_pending_${device_code}`,
        JSON.stringify(authData),
        { expirationTtl: Math.ceil((authData.expires_at - Date.now()) / 1000) }
      );
      
      return new Response(
        JSON.stringify({ error: "authorization_pending" }),
        {
          status: 401,
          headers: { "Content-Type": "application/json" }
        }
      );
    }
  } catch (error) {
    console.error("Claude auth poll error:", error);
    return new Response(
      JSON.stringify({ error: "Failed to poll Claude authentication" }),
      {
        status: 500,
        headers: { "Content-Type": "application/json" }
      }
    );
  }
}