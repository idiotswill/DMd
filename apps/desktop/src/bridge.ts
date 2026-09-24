import { invoke } from '@tauri-apps/api/core';

export interface DesktopStatus {
  appName: string;
  version: string;
  runtimeReady: boolean;
  message: string;
}

export async function loadDesktopStatus(): Promise<DesktopStatus> {
  return invoke<DesktopStatus>('desktop_status');
}
