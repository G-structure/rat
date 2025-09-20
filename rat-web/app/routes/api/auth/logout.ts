import type { APIEvent } from "solid-start/api";

export const onRequestPost = async (event: APIEvent) => {
  // Extract session ID from cookie
  const cookieHeader = event.request.headers.get("Cookie");
  const sid = cookieHeader?.match(/sid=([^;]+)/)?.[1];
  
  if (sid) {
    const { SESSIONS } = event.env as any;
    if (SESSIONS) {
      // Delete session from KV
      await SESSIONS.delete(`session:${sid}`);
    }
  }
  
  // Clear session cookie
  const headers = new Headers({ "Content-Type": "application/json" });
  headers.append(
    "Set-Cookie",
    "sid=; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=0"
  );
  
  return new Response(
    JSON.stringify({ ok: true }),
    { headers }
  );
};