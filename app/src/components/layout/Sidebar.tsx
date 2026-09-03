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
import { NavLink, useLocation } from 'react-router-dom'

import { BrandMark } from '@/components/layout/BrandMark'
import { cn } from '@/lib/utils'
import { useKagerouStore } from '@/store/kagerou-store'
import type { RouteKey } from '@/types/kagerou'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'

const navigation: Array<{ key: RouteKey; label: string; to: string; icon: typeof LayoutDashboard }> = [
  { key: 'dashboard', label: 'Dashboard', to: '/dashboard', icon: LayoutDashboard },
  { key: 'groups', label: 'Groups', to: '/groups', icon: Server },
  { key: 'sources', label: 'Sources', to: '/sources', icon: Rss },
  { key: 'routing-rules', label: 'Routing rules', to: '/routing-rules', icon: Route },
  { key: 'logs', label: 'Logs', to: '/logs', icon: FileText },
  { key: 'settings', label: 'Settings', to: '/settings', icon: Settings },
]

export function Sidebar() {
  const location = useLocation()
  const collapsed = useKagerouStore((state) => state.sidebarCollapsed)
  const toggleSidebar = useKagerouStore((state) => state.toggleSidebar)

  return (
    <aside
      aria-label="Primary navigation"
      className={cn(
        'group/sidebar sticky top-0 flex h-screen max-h-screen min-h-screen shrink-0 flex-col overflow-y-auto border-r border-hairline bg-sidebar px-3 py-6 transition-[width,padding] duration-200',
        collapsed ? 'w-[68px] px-3' : 'w-[216px]',
        'max-[960px]:w-[68px] max-[960px]:px-3',
      )}
    >
      <div className={cn('mb-11 flex items-center gap-3 px-3', collapsed && 'justify-center px-0', 'max-[960px]:justify-center max-[960px]:px-0')}>
        <BrandMark />
        <span className={cn('min-w-0', collapsed && 'hidden', 'max-[960px]:hidden')}>
          <span className="type-display block text-[18px] leading-none text-primary">Kagerou</span>
          <span className="mt-2 block text-[9px] uppercase tracking-[0.2em] text-muted-copy">Instrument panel</span>
        </span>
      </div>

      <nav aria-label="Primary navigation" className="flex flex-col gap-1">
        {navigation.map(({ label, to, icon: Icon }) => {
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
              <Tooltip key={label}>
                <TooltipTrigger asChild>{link}</TooltipTrigger>
                <TooltipContent side="right">{label}</TooltipContent>
              </Tooltip>
            )
          }

          return <span key={label}>{link}</span>
        })}
      </nav>

      <div className="mt-auto pt-8">
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                aria-expanded={!collapsed}
                aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
                className={cn(
                  'flex h-10 w-full items-center gap-3 rounded-lg px-3 text-left text-[13px] text-muted-copy transition-colors duration-150 hover:bg-row-hover hover:text-primary focus-visible:focus-ring',
                  collapsed && 'justify-center px-0',
                  'max-[960px]:justify-center max-[960px]:px-0',
                )}
                onClick={toggleSidebar}
                type="button"
              >
                {collapsed ? <PanelLeftOpen aria-hidden="true" className="size-[18px]" strokeWidth={1.7} /> : <PanelLeftClose aria-hidden="true" className="size-[18px]" strokeWidth={1.7} />}
                <span className={cn(collapsed && 'hidden', 'max-[960px]:hidden')}>{collapsed ? 'Expand sidebar' : 'Collapse sidebar'}</span>
              </button>
            </TooltipTrigger>
            <TooltipContent side="right">{collapsed ? 'Expand sidebar' : 'Collapse sidebar'}</TooltipContent>
          </Tooltip>
        </TooltipProvider>
      </div>
    </aside>
  )
}
