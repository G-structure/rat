import type { APIEvent } from "solid-start/api";

export const onRequestPost = async (event: APIEvent) => {
  const { GITHUB_CLIENT_ID, GITHUB_CLIENT_SECRET, APP_BASE_URL } = event.env as any;
  
  if (!GITHUB_CLIENT_ID || !GITHUB_CLIENT_SECRET) {
    return new Response(
      JSON.stringify({ error: "Server configuration error" }),
      {
        status: 500,
        headers: { "Content-Type": "application/json" }
      }
    );
  }
  
  try {
    const body = await event.request.json();
    const { device_code } = body;
    
    if (!device_code) {
      return new Response(
        JSON.stringify({ error: "Device code required" }),
        {
          status: 400,
          headers: { "Content-Type": "application/json" }
        }
      );
    }
    
    // Check with GitHub for authorization status
    const tokenResponse = await fetch("https://github.com/login/oauth/access_token", {
      method: "POST",
      headers: {
        "Content-Type": "application/x-www-form-urlencoded",
        "Accept": "application/json"
      },
      body: new URLSearchParams({
        client_id: GITHUB_CLIENT_ID,
        client_secret: GITHUB_CLIENT_SECRET,
        device_code,
        grant_type: "urn:ietf:params:oauth:grant-type:device_code"
      })
    });
    
    const tokenData = await tokenResponse.json();
    
    // Handle different response states
    if (tokenData.error === "authorization_pending") {
      // User hasn't authorized yet
      return new Response("Authorization pending", { status: 428 });
    }
    
    if (tokenData.error === "slow_down") {
      // Too many requests, client should slow down
      return new Response("Slow down", { status: 429 });
    }
    
    if (tokenData.error === "expired_token") {
      // Device code has expired
      return new Response("Device code expired", { status: 400 });
    }
    
    if (tokenData.error === "access_denied") {
      // User denied the authorization
      return new Response("Access denied", { status: 403 });
    }
    
    if (tokenData.error) {
      // Other errors
      return new Response(tokenData.error_description || tokenData.error, { 
        status: 400 
      });
    }
    
    // Success! We have an access token
    const { access_token } = tokenData;
    
    // Fetch user information
    const userResponse = await fetch("https://api.github.com/user", {
      headers: {
        "Authorization": `Bearer ${access_token}`,
        "User-Agent": "RAT-Mobile-IDE"
      }
    });
    
    if (!userResponse.ok) {
      throw new Error("Failed to fetch user data");
    }
    
    const userData = await userResponse.json();
    
    // Create session
    const sessionId = crypto.randomUUID();
    const { SESSIONS } = event.env as any;
    
    if (SESSIONS) {
      // Store session data
      await SESSIONS.put(
        `session:${sessionId}`,
        JSON.stringify({
          token: access_token,
          user: {
            id: userData.id,
            login: userData.login,
            name: userData.name,
            email: userData.email,
            avatar_url: userData.avatar_url
          },
          created: Date.now()
        }),
        { expirationTtl: 60 * 60 * 24 * 7 } // 7 days
      );
      
      // Clean up device code
      await SESSIONS.delete(`device:${device_code}`);
    }
    
    // Set session cookie
    const headers = new Headers({ "Content-Type": "application/json" });
    headers.append(
      "Set-Cookie",
      `sid=${sessionId}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=${60 * 60 * 24 * 7}`
    );
    
    return new Response(
      JSON.stringify({ 
        ok: true, 
        user: {
          login: userData.login,
          name: userData.name,
          avatar_url: userData.avatar_url
        }
      }),
      { headers }
    );
  } catch (error) {
    console.error("Device flow poll error:", error);
    return new Response(
      JSON.stringify({ error: "Authorization failed" }),
      {
        status: 500,
        headers: { "Content-Type": "application/json" }
      }
    );
  }
};