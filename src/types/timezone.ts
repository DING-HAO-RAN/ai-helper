export interface ProxyGeoInfo {
  ip: string;
  country: string;
  country_code: string;
  city: string;
  timezone_id: string;
  windows_tz_name: string;
  utc_offset_str: string;
  isp: string;
}

export interface ChatGPTStatus {
  is_running: boolean;
  pids: number[];
  cdp_available: boolean;
  active_timezone: string | null;
  app_path: string | null;
  system_timezone: string;
}

export interface TimezonePreset {
  id: string;
  label: string;
  windows_name: string;
  utc_offset: string;
  region: string;
}
