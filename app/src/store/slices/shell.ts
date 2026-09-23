import i18n from '@/i18n'
import { backendErrorMessage } from '@/lib/errors'
import { kagerouApi } from '@/lib/tauri-api'
import type { UpdateDownload, UpdateInfo } from '@/types/kagerou'

import { applySnapshot, report, type Slice } from '../shared'

export interface ShellSlice {
  hydrated: boolean
  /** Why the first read of the backend failed, if it did. */
  hydrateError: { message: string; dataDir: string } | null
  sidebarCollapsed: boolean
  updateAvailable: UpdateInfo | null
  updateDownload: UpdateDownload
  hydrate: () => Promise<void>
  toggleSidebar: () => void
  downloadUpdate: () => Promise<void>
  installUpdate: () => Promise<void>
}

/** The window itself: whether it has anything to draw yet, and the two bits
 * of chrome around whatever it is drawing. */
export const createShellSlice: Slice<ShellSlice> = (set, get) => ({
  hydrated: false,
  hydrateError: null,
  sidebarCollapsed: false,
  updateAvailable: null,
  updateDownload: { phase: 'idle' },

  hydrate: async () => {
    let snapshot
    try {
      snapshot = await kagerouApi.getAppState()
    } catch (error) {
      // Without this the window paints nothing at all: App renders null until
      // `hydrated` flips, and the rejection lands in a console nobody opens.
      const dataDir = await kagerouApi.appDataDir().catch(() => '')
      set({
        hydrated: true,
        hydrateError: {
          dataDir,
          message: backendErrorMessage(error, i18n.t('common:feedback.startupFailed')),
        },
      })
      return
    }
    set({ ...applySnapshot(snapshot), hydrated: true })
    // Deliberately not awaited: a slow or unreachable GitHub must not hold
    // up the first paint, and the command never rejects.
    void kagerouApi.checkForUpdate().then((updateAvailable) => set({ updateAvailable }))
    // A reload mid-session lands here with the tunnel already up, and the
    // connection-changed event that would have asked has long since fired.
    if (snapshot.connected) void get().refreshExitLocation()
  },

  toggleSidebar: () => set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),

  downloadUpdate: async () => {
    if (get().updateDownload.phase !== 'idle') return
    set({ updateDownload: { phase: 'downloading', downloaded: 0, total: null } })
    try {
      await kagerouApi.downloadUpdate()
      set({ updateDownload: { phase: 'ready' } })
    } catch (error) {
      report(error, 'common:feedback.updateDownloadFailed')
      set({ updateDownload: { phase: 'idle' } })
    }
  },

  installUpdate: async () => {
    if (get().updateDownload.phase !== 'ready') return
    set({ updateDownload: { phase: 'installing' } })
    try {
      await kagerouApi.installUpdate()
    } catch (error) {
      // The backend keeps the download on a failed install, so the restart
      // button can simply be offered again.
      report(error, 'common:feedback.updateInstallFailed')
      set({ updateDownload: { phase: 'ready' } })
    }
  },
})
