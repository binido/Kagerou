import { vi } from 'vitest'

import type { ExitLocation } from '@/types/kagerou'

import { emptySnapshot } from './test-fixtures'

/** One mock of the backend bridge for every store test file.
 *
 * Shared rather than declared per file because the store subscribes to all
 * eight events on start-up: a file testing one action would still have to
 * list every listener for the subscription not to throw. */
export const kagerouApiMock = {
  getAppState: vi.fn(),
  appDataDir: vi.fn(),
  checkForUpdate: vi.fn(),
  connect: vi.fn(),
  disconnect: vi.fn(),
  lookupExitLocation: vi.fn(async (): Promise<ExitLocation | null> => null),
  listConnections: vi.fn(),
  closeConnection: vi.fn(),
  closeAllConnections: vi.fn(),
  selectProfile: vi.fn(),
  renameProfile: vi.fn(),
  deleteProfile: vi.fn(),
  moveProfileToGroup: vi.fn(),
  moveProfile: vi.fn(),
  reorderProfiles: vi.fn(),
  runProfileTest: vi.fn(),
  startGroupTest: vi.fn(),
  cancelGroupTest: vi.fn(),
  clearGroupTestResults: vi.fn(),
  deleteUnavailableProfiles: vi.fn(),
  setProfileGroupOpen: vi.fn(),
  addProfileGroup: vi.fn(),
  renameProfileGroup: vi.fn(),
  readClipboardText: vi.fn(),
  importFromText: vi.fn(),
  updateSource: vi.fn(),
  refreshSource: vi.fn(),
  deleteSubscription: vi.fn(),
  openSupportUrl: vi.fn(),
  setPreset: vi.fn(),
  selectRule: vi.fn(),
  updateRule: vi.fn(),
  addRoutingRule: vi.fn(),
  deleteRoutingRule: vi.fn(),
  analyzeRuleMatch: vi.fn(),
  setTheme: vi.fn(),
  updateSettings: vi.fn(),
  onConnectionChanged: vi.fn(),
  onTraffic: vi.fn(),
  onLog: vi.fn(),
  onTestProgress: vi.fn(),
  onTestFinished: vi.fn(),
  onTrayToggleConnection: vi.fn(),
  onTraySelectProfile: vi.fn(),
  onCrashed: vi.fn(),
}

const LISTENERS = [
  kagerouApiMock.onConnectionChanged,
  kagerouApiMock.onTraffic,
  kagerouApiMock.onLog,
  kagerouApiMock.onTestProgress,
  kagerouApiMock.onTestFinished,
  kagerouApiMock.onTrayToggleConnection,
  kagerouApiMock.onTraySelectProfile,
  kagerouApiMock.onCrashed,
]

// Fire-and-forget mutations call `.catch()` on the invoke promise, so every
// action mock needs to resolve to *something* by default even in tests that
// don't care about its outcome.
const FIRE_AND_FORGET = [
  kagerouApiMock.setProfileGroupOpen,
  kagerouApiMock.setPreset,
  kagerouApiMock.selectRule,
  kagerouApiMock.updateRule,
  kagerouApiMock.setTheme,
  kagerouApiMock.updateSettings,
]

export const resetApiMock = () => {
  Object.values(kagerouApiMock).forEach((fn) => fn.mockReset())
  kagerouApiMock.getAppState.mockResolvedValue(emptySnapshot)
  kagerouApiMock.appDataDir.mockResolvedValue('/data/kagerou')
  kagerouApiMock.checkForUpdate.mockResolvedValue(null)
  LISTENERS.forEach((fn) => fn.mockResolvedValue(() => {}))
  FIRE_AND_FORGET.forEach((fn) => fn.mockResolvedValue(undefined))
}
