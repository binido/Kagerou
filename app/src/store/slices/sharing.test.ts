import { beforeEach, describe, expect, it, vi } from 'vitest'

import { toast } from 'sonner'

vi.mock('@/lib/tauri-api', async () => ({
  kagerouApi: (await import('../test-api')).kagerouApiMock,
}))
vi.mock('@/themes/runtime', () => ({ persistThemeId: vi.fn() }))
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn(), loading: vi.fn() } }))

import en from '@/locales/en/common.json'

import { kagerouApiMock as api, resetApiMock } from '../test-api'

const { useKagerouStore } = await import('../kagerou-store')

const writeText = vi.fn()

beforeEach(() => {
  resetApiMock()
  vi.mocked(toast.error).mockClear()
  vi.mocked(toast.success).mockClear()
  writeText.mockReset()
  writeText.mockResolvedValue(undefined)
  vi.stubGlobal('navigator', { clipboard: { writeText } })
})

describe('sharing VPNs', () => {
  it('copies the backend-built links one per line', async () => {
    api.exportLinks.mockResolvedValue(['vless://a#A', 'trojan://b#B'])

    const ok = await useKagerouStore.getState().copyProfileLinks(['p1', 'p2'])

    expect(ok).toBe(true)
    expect(api.exportLinks).toHaveBeenCalledWith(['p1', 'p2'])
    expect(writeText).toHaveBeenCalledWith('vless://a#A\ntrojan://b#B')
    expect(toast.success).toHaveBeenCalledOnce()
  })

  it('says so when the clipboard refuses', async () => {
    api.exportLinks.mockResolvedValue(['vless://a#A'])
    writeText.mockRejectedValue(new Error('denied'))

    expect(await useKagerouStore.getState().copyProfileLinks(['p1'])).toBe(false)
    expect(toast.error).toHaveBeenCalledOnce()
    expect(toast.success).not.toHaveBeenCalled()
  })

  it('stays quiet when the save dialog is closed', async () => {
    api.saveLinksToFile.mockResolvedValue(false)

    expect(await useKagerouStore.getState().saveProfileLinks(['p1'], 'Work')).toBe(false)
    expect(api.saveLinksToFile).toHaveBeenCalledWith(['p1'], 'Work')
    expect(toast.success).not.toHaveBeenCalled()
    expect(toast.error).not.toHaveBeenCalled()
  })

  it('reports a file that could not be written in the backend’s terms', async () => {
    api.saveLinksToFile.mockRejectedValue({ code: 'fileWrite', detail: '/x: denied' })

    expect(await useKagerouStore.getState().saveProfileLinks(['p1'], 'Work')).toBe(false)
    expect(toast.error).toHaveBeenCalledWith(en.errors.fileWrite)
  })

  it('hands back no code when the QR cannot be made', async () => {
    api.profileQrSvg.mockRejectedValue({ code: 'notFound', detail: 'gone' })

    expect(await useKagerouStore.getState().loadProfileQr('p1')).toBeNull()
    expect(toast.error).toHaveBeenCalledWith(en.errors.notFound)
  })
})
