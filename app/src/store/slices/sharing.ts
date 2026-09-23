import { toast } from 'sonner'

import i18n from '@/i18n'
import { kagerouApi } from '@/lib/tauri-api'

import { report, type Slice } from '../shared'

export interface SharingSlice {
  /** Puts the profiles' links on the clipboard, one per line. */
  copyProfileLinks: (ids: string[]) => Promise<boolean>
  /** Resolves to `true` once written, `false` when the dialog was closed. */
  saveProfileLinks: (ids: string[], label: string) => Promise<boolean>
  loadProfileQr: (id: string) => Promise<string | null>
}

/** Getting VPNs out: as links, a file of links, or a QR code. The links are
 * built by the backend, so a renamed VPN goes out under its current name. */
export const createSharingSlice: Slice<SharingSlice> = () => ({
  copyProfileLinks: async (ids) => {
    try {
      const links = await kagerouApi.exportLinks(ids)
      await navigator.clipboard.writeText(links.join('\n'))
      toast.success(i18n.t('profiles:share.copied', { count: links.length }))
      return true
    } catch (error) {
      report(error, 'profiles:share.copyFailed')
      return false
    }
  },

  saveProfileLinks: async (ids, label) => {
    try {
      const saved = await kagerouApi.saveLinksToFile(ids, label)
      if (saved) toast.success(i18n.t('profiles:share.saved', { count: ids.length }))
      return saved
    } catch (error) {
      report(error, 'profiles:share.saveFailed')
      return false
    }
  },

  loadProfileQr: async (id) => {
    try {
      return await kagerouApi.profileQrSvg(id)
    } catch (error) {
      report(error, 'profiles:share.qrFailed')
      return null
    }
  },
})
