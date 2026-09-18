export interface TokenDisplayView {
  id: string;
  alias: string;
  masked_token: string;
  note: string;
  is_default: boolean;
  created_at: string;
  updated_at: string;
}

export interface GitHubUserInfo {
  login: string;
  name: string | null;
  avatar_url: string | null;
  public_repos: number | null;
}
