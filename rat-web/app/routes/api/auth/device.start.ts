import type { APIEvent } from "solid-start/api";

export const onRequestPost = async (event: APIEvent) => {
  const { GITHUB_CLIENT_ID } = event.env as any;
  
  if (!GITHUB_CLIENT_ID) {
    return new Response(
      JSON.stringify({ error: "Server configuration error" }), 
      { 
        status: 500,
        headers: { "Content-Type": "application/json" }
      }
    );
  }
  
  try {
    // Request device code from GitHub
    const response = await fetch("https://github.com/login/device/code", {
      method: "POST",
      headers: { 
        "Content-Type": "application/x-www-form-urlencoded",
        "Accept": "application/json"
      },
      body: new URLSearchParams({
        client_id: GITHUB_CLIENT_ID,
        scope: "repo read:user"
      })
    });
    
    if (!response.ok) {
      throw new Error(`GitHub API error: ${response.status}`);
    }
    
    const data = await response.json();
    const { device_code, user_code, verification_uri, expires_in, interval } = data;
    
    // Store device code in KV with expiration
    const { SESSIONS } = event.env as any;
    if (SESSIONS) {
      await SESSIONS.put(
        `device:${device_code}`,
        JSON.stringify({ 
          created: Date.now(),
          status: "pending"
        }),
        { expirationTtl: expires_in }
      );
    }
    
    // Return device authorization details to client
    return new Response(
      JSON.stringify({
        device_code,
        user_code,
        verification_uri,
        expires_in,
        interval: interval || 5
      }),
      {
        headers: { "Content-Type": "application/json" }
      }
    );
  } catch (error) {
    console.error("Device flow start error:", error);
    return new Response(
      JSON.stringify({ error: "Failed to start device flow" }),
      {
        status: 500,
        headers: { "Content-Type": "application/json" }
      }
    );
  }
};