import type { APIEvent } from "solid-start/api";

async function getSession(event: APIEvent) {
  const cookieHeader = event.request.headers.get("Cookie");
  const sid = cookieHeader?.match(/sid=([^;]+)/)?.[1];
  
  if (!sid) return null;
  
  const { SESSIONS } = event.env as any;
  if (!SESSIONS) return null;
  
  return await SESSIONS.get(`session:${sid}`, "json");
}

export const onRequestGet = async (event: APIEvent) => {
  const session = await getSession(event);
  
  if (!session) {
    return new Response(JSON.stringify({ error: "Unauthorized" }), {
      status: 401,
      headers: { "Content-Type": "application/json" }
    });
  }
  
  try {
    // Get query parameters
    const url = new URL(event.request.url);
    const page = url.searchParams.get("page") || "1";
    const per_page = url.searchParams.get("per_page") || "30";
    const sort = url.searchParams.get("sort") || "updated";
    
    // Fetch repositories from GitHub
    const response = await fetch(
      `https://api.github.com/user/repos?page=${page}&per_page=${per_page}&sort=${sort}`,
      {
        headers: {
          "Authorization": `Bearer ${session.token}`,
          "User-Agent": "RAT-Mobile-IDE",
          "Accept": "application/vnd.github.v3+json"
        }
      }
    );
    
    if (!response.ok) {
      throw new Error(`GitHub API error: ${response.status}`);
    }
    
    const repos = await response.json();
    
    // Transform the response to include only necessary fields
    const simplifiedRepos = repos.map((repo: any) => ({
      id: repo.id,
      name: repo.name,
      full_name: repo.full_name,
      owner: {
        login: repo.owner.login,
        avatar_url: repo.owner.avatar_url
      },
      description: repo.description,
      language: repo.language,
      stargazers_count: repo.stargazers_count,
      forks_count: repo.forks_count,
      open_issues_count: repo.open_issues_count,
      default_branch: repo.default_branch,
      updated_at: repo.updated_at,
      private: repo.private,
      topics: repo.topics || []
    }));
    
    // Get link header for pagination
    const linkHeader = response.headers.get("Link");
    const hasNextPage = linkHeader?.includes('rel="next"') || false;
    const hasPrevPage = linkHeader?.includes('rel="prev"') || false;
    
    return new Response(
      JSON.stringify({
        repos: simplifiedRepos,
        pagination: {
          page: parseInt(page),
          per_page: parseInt(per_page),
          has_next: hasNextPage,
          has_prev: hasPrevPage
        }
      }),
      {
        headers: { "Content-Type": "application/json" }
      }
    );
  } catch (error) {
    console.error("Repos endpoint error:", error);
    return new Response(
      JSON.stringify({ error: "Failed to fetch repositories" }),
      {
        status: 500,
        headers: { "Content-Type": "application/json" }
      }
    );
  }
};

// Get a specific repository
export const onRequestGetWithParams = async (event: APIEvent & { params: { owner: string, repo: string } }) => {
  const session = await getSession(event);
  
  if (!session) {
    return new Response(JSON.stringify({ error: "Unauthorized" }), {
      status: 401,
      headers: { "Content-Type": "application/json" }
    });
  }
  
  const { owner, repo } = event.params;
  
  try {
    const response = await fetch(
      `https://api.github.com/repos/${owner}/${repo}`,
      {
        headers: {
          "Authorization": `Bearer ${session.token}`,
          "User-Agent": "RAT-Mobile-IDE",
          "Accept": "application/vnd.github.v3+json"
        }
      }
    );
    
    if (!response.ok) {
      throw new Error(`GitHub API error: ${response.status}`);
    }
    
    const data = await response.json();
    
    return new Response(JSON.stringify(data), {
      headers: { "Content-Type": "application/json" }
    });
  } catch (error) {
    console.error("Repo endpoint error:", error);
    return new Response(
      JSON.stringify({ error: "Failed to fetch repository" }),
      {
        status: 500,
        headers: { "Content-Type": "application/json" }
      }
    );
  }
};