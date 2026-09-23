import type { AppSnapshot } from '@/lib/tauri-api'
import type { Profile } from '@/types/kagerou'

/** A backend that has nothing in it yet: what most tests hydrate from
 * before setting up the one thing they are about. */
export const emptySnapshot: AppSnapshot = {
  connected: false,
  connectedSince: null,
  activeProfileId: '',
  profiles: [],
  profileGroups: [],
  sources: [],
  routingPresets: [],
  routingRules: [],
  settings: {
    theme: 'catppuccin-mocha',
    language: 'en',
    startup: false,
    geoLookup: true,
    tunMode: false,
    systemProxy: false,
    autoConnect: false,
    tunInterface: 'utun / tun0',
    autoUpdateSubscriptions: false,
    subscriptionUpdateInterval: '30',
    customSubscriptionUpdateMinutes: 60,
    groupSort: 'ping',
    logLevel: 'info',
    testUrl: 'http://www.gstatic.com/generate_204',
    mixedPort: 2080,
    clashApiPort: 9090,
  },
}

export const profile = (overrides: Partial<Profile> = {}): Profile => ({
  id: 'p1',
  name: 'P1',
  region: 'us',
  protocol: 'VLESS',
  origin: 'local',
  groupId: 'default',
  selected: false,
  url: { kind: 'notTested', tone: 'muted' },
  key: 'vless://p1',
  ...overrides,
})
