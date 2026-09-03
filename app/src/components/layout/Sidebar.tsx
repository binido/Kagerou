import {
  FileText,
  LayoutDashboard,
  PanelLeftClose,
  PanelLeftOpen,
  Route,
  Rss,
  Server,
  Settings,
} from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { NavLink, useLocation } from 'react-router-dom'

import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { BrandMark } from '@/components/layout/BrandMark'
import { cn } from '@/lib/utils'
import { useKagerouStore } from '@/store/kagerou-store'
import type { RouteKey } from '@/types/kagerou'

type SidebarLabelKey =
  | 'sidebar.dashboard'
  | 'sidebar.groups'
  | 'sidebar.sources'
  | 'sidebar.routingRules'
  | 'sidebar.logs'
  | 'sidebar.settings'

const navigation: Array<{ key: RouteKey; labelKey: SidebarLabelKey; to: string; icon: typeof LayoutDashboard }> = [
  { key: 'dashboard', labelKey: 'sidebar.dashboard', to: '/dashboard', icon: LayoutDashboard },
  { key: 'groups', labelKey: 'sidebar.groups', to: '/groups', icon: Server },
  { key: 'sources', labelKey: 'sidebar.sources', to: '/sources', icon: Rss },
  { key: 'routing-rules', labelKey: 'sidebar.routingRules', to: '/routing-rules', icon: Route },
  { key: 'logs', labelKey: 'sidebar.logs', to: '/logs', icon: FileText },
  { key: 'settings', labelKey: 'sidebar.settings', to: '/settings', icon: Settings },
]

export function Sidebar() {
  const { t } = useTranslation('common')
  const location = useLocation()
  const collapsed = useKagerouStore((state) => state.sidebarCollapsed)
  const toggleSidebar = useKagerouStore((state) => state.toggleSidebar)

  return (
    <aside
      aria-label={t('sidebar.primaryNavigation')}
      className={cn(
        'group/sidebar sticky top-0 flex h-screen max-h-screen min-h-screen shrink-0 flex-col overflow-y-auto border-r border-hairline bg-sidebar px-3 py-6 transition-[width,padding] duration-200',
        collapsed ? 'w-[68px] px-3' : 'w-[216px]',
        'max-[960px]:w-[68px] max-[960px]:px-3',
      )}
    >
      <div className={cn('mb-11 flex items-center gap-3 px-3', collapsed && 'justify-center px-0', 'max-[960px]:justify-center max-[960px]:px-0')}>
        <BrandMark />
        <span className={cn('min-w-0', collapsed && 'hidden', 'max-[960px]:hidden')}>
          <span className="type-display block text-[18px] leading-none text-primary">{t('brand.name')}</span>
          <span className="mt-2 block text-[9px] uppercase tracking-[0.2em] text-muted-copy">{t('brand.tagline')}</span>
        </span>
      </div>

      <nav aria-label={t('sidebar.primaryNavigation')} className="flex flex-col gap-1">
        {navigation.map(({ labelKey, to, icon: Icon }) => {
          const label = t(labelKey) as string
          const isActive = location.pathname === to || (to === '/sources' && location.pathname === '/subscriptions')
          const link = (
            <NavLink
              aria-current={isActive ? 'page' : undefined}
              className={cn(
                'flex h-10 items-center gap-3 rounded-lg px-3 text-[13px] text-body transition-colors duration-150 hover:bg-row-hover hover:text-primary focus-visible:focus-ring',
                isActive && 'bg-selected font-medium text-primary',
                (collapsed || isActive) && 'max-[960px]:bg-transparent',
                collapsed && 'size-10 justify-center bg-transparent p-0',
                'max-[960px]:size-10 max-[960px]:justify-center max-[960px]:bg-transparent max-[960px]:p-0',
              )}
              to={to}
            >
              <span
                className={cn(
                  'flex shrink-0 items-center justify-center',
                  collapsed ? 'size-10 rounded-lg' : 'size-[18px]',
                  collapsed && isActive && 'bg-selected',
                  'max-[960px]:size-10 max-[960px]:rounded-lg',
                  isActive && 'max-[960px]:bg-selected',
                )}
              >
                <Icon aria-hidden="true" className="size-[18px] shrink-0" strokeWidth={1.7} />
              </span>
              <span className={cn(collapsed && 'hidden', 'max-[960px]:hidden')}>{label}</span>
            </NavLink>
          )

          if (collapsed) {
            return (
              <Tooltip key={to}>
                <TooltipTrigger asChild>{link}</TooltipTrigger>
                <TooltipContent side="right">{label}</TooltipContent>
              </Tooltip>
            )
          }

          return <span key={to}>{link}</span>
        })}
      </nav>

      <div className="mt-auto pt-8">
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                aria-expanded={!collapsed}
                aria-label={collapsed ? t('sidebar.expand') : t('sidebar.collapse')}
                className={cn(
                  'flex h-10 w-full items-center gap-3 rounded-lg px-3 text-left text-[13px] text-muted-copy transition-colors duration-150 hover:bg-row-hover hover:text-primary focus-visible:focus-ring',
                  collapsed && 'justify-center px-0',
                  'max-[960px]:justify-center max-[960px]:px-0',
                )}
                onClick={toggleSidebar}
                type="button"
              >
                {collapsed ? <PanelLeftOpen aria-hidden="true" className="size-[18px]" strokeWidth={1.7} /> : <PanelLeftClose aria-hidden="true" className="size-[18px]" strokeWidth={1.7} />}
                <span className={cn(collapsed && 'hidden', 'max-[960px]:hidden')}>{collapsed ? t('sidebar.expand') : t('sidebar.collapse')}</span>
              </button>
            </TooltipTrigger>
            <TooltipContent side="right">{collapsed ? t('sidebar.expand') : t('sidebar.collapse')}</TooltipContent>
          </Tooltip>
        </TooltipProvider>
      </div>
    </aside>
  )
}
