// Mock API for development without backend
export const mockApi = {
  async startDeviceFlow() {
    // Simulate device flow
    return {
      device_code: "MOCK-DEVICE-CODE",
      user_code: "ABCD-1234",
      verification_uri: "https://github.com/login/device",
      expires_in: 900,
      interval: 5
    };
  },

  async pollDeviceFlow(device_code: string) {
    // Simulate successful auth after a delay
    await new Promise(resolve => setTimeout(resolve, 2000));
    return {
      ok: true,
      user: {
        login: "demo-user",
        name: "Demo User",
        avatar_url: "https://avatars.githubusercontent.com/u/1?v=4"
      }
    };
  },

  async getMe() {
    const mockUser = localStorage.getItem("mock-user");
    if (mockUser) {
      return JSON.parse(mockUser);
    }
    return { user: null };
  },

  async getRepos() {
    return {
      repos: [
        {
          id: 1,
          name: "demo-project",
          full_name: "demo-user/demo-project",
          owner: {
            login: "demo-user",
            avatar_url: "https://avatars.githubusercontent.com/u/1?v=4"
          },
          description: "A demo project for testing",
          language: "TypeScript",
          stargazers_count: 42,
          forks_count: 7,
          open_issues_count: 3,
          default_branch: "main",
          updated_at: new Date().toISOString(),
          private: false,
          topics: ["demo", "typescript"]
        },
        {
          id: 2,
          name: "another-project",
          full_name: "demo-user/another-project",
          owner: {
            login: "demo-user",
            avatar_url: "https://avatars.githubusercontent.com/u/1?v=4"
          },
          description: "Another demo project",
          language: "JavaScript",
          stargazers_count: 15,
          forks_count: 2,
          open_issues_count: 1,
          default_branch: "main",
          updated_at: new Date(Date.now() - 86400000).toISOString(),
          private: false,
          topics: ["javascript", "web"]
        }
      ],
      pagination: {
        page: 1,
        per_page: 30,
        has_next: false,
        has_prev: false
      }
    };
  },

  async getFileContents(owner: string, repo: string, branch: string, path: string) {
    if (!path) {
      // Return directory listing
      return [
        { name: "src", path: "src", type: "dir", size: 0 },
        { name: "README.md", path: "README.md", type: "file", size: 1234 },
        { name: "package.json", path: "package.json", type: "file", size: 567 },
        { name: ".gitignore", path: ".gitignore", type: "file", size: 89 }
      ];
    } else if (path === "README.md") {
      // Return file content
      return {
        name: "README.md",
        path: "README.md",
        sha: "abc123",
        size: 1234,
        type: "file",
        encoding: "base64",
        content: `# Demo Project

This is a demo project for testing the RAT Mobile IDE.

## Features
- Mobile-first design
- GitHub integration
- AI-powered editing

## Usage
\`\`\`bash
npm install
npm run dev
\`\`\`
`
      };
    }
    return [];
  }
};