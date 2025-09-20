import type { APIEvent } from "solid-start/api";

export const onRequestGet = async (event: APIEvent) => {
  // Extract session ID from cookie
  const cookieHeader = event.request.headers.get("Cookie");
  const sid = cookieHeader?.match(/sid=([^;]+)/)?.[1];
  
  if (!sid) {
    return new Response(
      JSON.stringify({ user: null }),
      {
        status: 401,
        headers: { "Content-Type": "application/json" }
      }
    );
  }
  
  const { SESSIONS } = event.env as any;
  if (!SESSIONS) {
    return new Response(
      JSON.stringify({ error: "Server configuration error" }),
      {
        status: 500,
        headers: { "Content-Type": "application/json" }
      }
    );
  }
  
  try {
    // Fetch session from KV
    const sessionData = await SESSIONS.get(`session:${sid}`, "json");
    
    if (!sessionData) {
      return new Response(
        JSON.stringify({ user: null }),
        {
          status: 401,
          headers: { "Content-Type": "application/json" }
        }
      );
    }
    
    // Optionally refresh user data from GitHub
    const { token, user } = sessionData;
    
    // For now, just return cached user data
    // In production, you might want to periodically refresh this
    return new Response(
      JSON.stringify({
        user,
        credits: 1000, // Mock credits for now
        session: {
          created: sessionData.created,
          expires: sessionData.created + (60 * 60 * 24 * 7 * 1000) // 7 days
        }
      }),
      {
        headers: { "Content-Type": "application/json" }
      }
    );
  } catch (error) {
    console.error("Me endpoint error:", error);
    return new Response(
      JSON.stringify({ error: "Failed to fetch user data" }),
      {
        status: 500,
        headers: { "Content-Type": "application/json" }
      }
    );
  }
};