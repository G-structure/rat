import type { APIEvent } from "solid-start/api";

async function getSession(event: APIEvent) {
  const cookieHeader = event.request.headers.get("Cookie");
  const sid = cookieHeader?.match(/sid=([^;]+)/)?.[1];
  
  if (!sid) return null;
  
  const { SESSIONS } = event.env as any;
  if (!SESSIONS) return null;
  
  return await SESSIONS.get(`session:${sid}`, "json");
}

export const onRequestGet = async (event: APIEvent & { params: { path: string } }) => {
  const session = await getSession(event);
  
  if (!session) {
    return new Response(JSON.stringify({ error: "Unauthorized" }), {
      status: 401,
      headers: { "Content-Type": "application/json" }
    });
  }
  
  // Parse the path: owner/repo/branch/...filePath
  const pathParts = event.params.path.split("/");
  if (pathParts.length < 3) {
    return new Response(
      JSON.stringify({ error: "Invalid path format" }), 
      {
        status: 400,
        headers: { "Content-Type": "application/json" }
      }
    );
  }
  
  const owner = pathParts[0];
  const repo = pathParts[1];
  const branch = pathParts[2];
  const filePath = pathParts.slice(3).join("/");
  
  try {
    // GitHub Contents API endpoint
    const apiPath = filePath 
      ? `https://api.github.com/repos/${owner}/${repo}/contents/${filePath}?ref=${branch}`
      : `https://api.github.com/repos/${owner}/${repo}/contents?ref=${branch}`;
    
    const response = await fetch(apiPath, {
      headers: {
        "Authorization": `Bearer ${session.token}`,
        "User-Agent": "RAT-Mobile-IDE",
        "Accept": "application/vnd.github.v3+json"
      }
    });
    
    if (!response.ok) {
      if (response.status === 404) {
        return new Response(
          JSON.stringify({ error: "File or directory not found" }),
          {
            status: 404,
            headers: { "Content-Type": "application/json" }
          }
        );
      }
      throw new Error(`GitHub API error: ${response.status}`);
    }
    
    const data = await response.json();
    
    // Check if it's a file or directory
    if (Array.isArray(data)) {
      // Directory listing
      const items = data.map((item: any) => ({
        name: item.name,
        path: item.path,
        type: item.type,
        size: item.size,
        sha: item.sha,
        download_url: item.download_url
      }));
      
      // Sort directories first, then files
      items.sort((a: any, b: any) => {
        if (a.type === b.type) return a.name.localeCompare(b.name);
        return a.type === "dir" ? -1 : 1;
      });
      
      return new Response(JSON.stringify(items), {
        headers: { "Content-Type": "application/json" }
      });
    } else {
      // Single file
      const fileInfo: any = {
        name: data.name,
        path: data.path,
        sha: data.sha,
        size: data.size,
        type: data.type,
        encoding: data.encoding
      };
      
      // Decode content if it's base64 encoded
      if (data.encoding === "base64" && data.content) {
        try {
          // Remove newlines from base64 string
          const cleanContent = data.content.replace(/\n/g, "");
          // Decode base64
          const decoded = atob(cleanContent);
          fileInfo.content = decoded;
        } catch (e) {
          console.error("Failed to decode file content:", e);
          fileInfo.content = data.content;
          fileInfo.decode_error = true;
        }
      } else if (data.content) {
        fileInfo.content = data.content;
      }
      
      // For large files, GitHub returns a download_url instead of content
      if (!fileInfo.content && data.download_url) {
        // Fetch the raw content
        const rawResponse = await fetch(data.download_url, {
          headers: {
            "Authorization": `Bearer ${session.token}`,
            "User-Agent": "RAT-Mobile-IDE"
          }
        });
        
        if (rawResponse.ok) {
          fileInfo.content = await rawResponse.text();
        }
      }
      
      return new Response(JSON.stringify(fileInfo), {
        headers: { "Content-Type": "application/json" }
      });
    }
  } catch (error) {
    console.error("Files endpoint error:", error);
    return new Response(
      JSON.stringify({ error: "Failed to fetch content" }),
      {
        status: 500,
        headers: { "Content-Type": "application/json" }
      }
    );
  }
};

// Update file content
export const onRequestPut = async (event: APIEvent & { params: { path: string } }) => {
  const session = await getSession(event);
  
  if (!session) {
    return new Response(JSON.stringify({ error: "Unauthorized" }), {
      status: 401,
      headers: { "Content-Type": "application/json" }
    });
  }
  
  // Parse the path
  const pathParts = event.params.path.split("/");
  if (pathParts.length < 4) {
    return new Response(
      JSON.stringify({ error: "Invalid path format" }), 
      {
        status: 400,
        headers: { "Content-Type": "application/json" }
      }
    );
  }
  
  const owner = pathParts[0];
  const repo = pathParts[1];
  const branch = pathParts[2];
  const filePath = pathParts.slice(3).join("/");
  
  try {
    const body = await event.request.json();
    const { content, message, sha } = body;
    
    if (!content || !message) {
      return new Response(
        JSON.stringify({ error: "Content and message are required" }),
        {
          status: 400,
          headers: { "Content-Type": "application/json" }
        }
      );
    }
    
    // Encode content to base64
    const encodedContent = btoa(content);
    
    const response = await fetch(
      `https://api.github.com/repos/${owner}/${repo}/contents/${filePath}`,
      {
        method: "PUT",
        headers: {
          "Authorization": `Bearer ${session.token}`,
          "User-Agent": "RAT-Mobile-IDE",
          "Accept": "application/vnd.github.v3+json",
          "Content-Type": "application/json"
        },
        body: JSON.stringify({
          message,
          content: encodedContent,
          branch,
          sha // Required for updates, optional for creates
        })
      }
    );
    
    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.message || `GitHub API error: ${response.status}`);
    }
    
    const result = await response.json();
    
    return new Response(
      JSON.stringify({
        commit: result.commit,
        content: {
          name: result.content.name,
          path: result.content.path,
          sha: result.content.sha,
          size: result.content.size
        }
      }),
      {
        headers: { "Content-Type": "application/json" }
      }
    );
  } catch (error) {
    console.error("File update error:", error);
    return new Response(
      JSON.stringify({ error: error.message || "Failed to update file" }),
      {
        status: 500,
        headers: { "Content-Type": "application/json" }
      }
    );
  }
};