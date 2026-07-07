import type { PolicyResponse, PresetName } from './api-contract';

export type AuthSession = {
  authType: string | null;
  userEmail: string | null;
  userName: string | null;
};

export type AuthState =
  | { kind: 'Checking' }
  | { kind: 'Authenticated'; session: AuthSession }
  | { kind: 'Unauthenticated' };

export type DashboardView = 'scan' | 'policy' | 'settings';

export type PolicyViewState =
  | { kind: 'Loading' }
  | { kind: 'Ready'; policy: PolicyResponse }
  | { kind: 'ErrorAuth'; message: string }
  | { kind: 'ErrorUnknown'; message: string };

export type ScanState =
  | 'Idle'
  | 'Running'
  | 'SuccessNoFindings'
  | 'Violation'
  | 'Incomplete'
  | 'ErrorAuth'
  | 'ErrorConfig'
  | 'ErrorExpired'
  | 'ErrorOOM'
  | 'ErrorUnknown';

export type ScanPreset = '' | PresetName;
