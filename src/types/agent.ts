export interface AgentFileInfo {
  path: string;
  filename: string;
  size_bytes: number;
  modified_at: string;
  preview: string;
  has_token_guide: boolean;
  tool_or_project: string;
}

export interface AgentIndexCache {
  updated_at: string;
  total_count: number;
  agents: AgentFileInfo[];
}

export interface InjectSummary {
  total: number;
  success: number;
  failed: number;
  details: {
    path: string;
    success: boolean;
    message: string;
  }[];
}
