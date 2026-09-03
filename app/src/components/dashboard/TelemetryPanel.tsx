import { Activity } from 'lucide-react'

import { Card } from '@/components/ui/card'
import { TelemetryChart } from '@/components/dashboard/TelemetryChart'
import type { TelemetryPoint } from '@/types/kagerou'

export function TelemetryPanel({ data }: { data: TelemetryPoint[] }) {
  return (
    <Card className="flex min-h-[584px] flex-col rounded-[10px] border border-hairline bg-surface p-0 shadow-none">
      <div className="border-b border-hairline px-6 py-6">
        <div className="flex items-center justify-between">
          <h2 className="type-eyebrow">Live telemetry</h2>
          <span className="type-data text-muted-copy">60 SEC</span>
        </div>
        <div className="mt-7 grid grid-cols-2 gap-4">
          <div>
            <p className="text-[12px] text-muted-copy">Download</p>
            <p className="type-display mt-1 whitespace-nowrap text-[29px] leading-none tracking-[-0.02em] text-primary">
              82.4 <span className="font-sans text-[12px] font-medium tracking-normal text-body">Mbps</span>
            </p>
          </div>
          <div>
            <p className="text-[12px] text-muted-copy">Upload</p>
            <p className="type-display mt-1 whitespace-nowrap text-[29px] leading-none tracking-[-0.02em] text-primary">
              18.7 <span className="font-sans text-[12px] font-medium tracking-normal text-body">Mbps</span>
            </p>
          </div>
        </div>
      </div>
      <div className="min-h-0 flex-1 px-6 py-6">
        <div className="flex items-center justify-between">
          <p className="text-[12px] text-body">Last 60 seconds</p>
          <span className="type-data text-muted-copy">Mbps</span>
        </div>
        <div className="mt-5 h-[202px] min-w-0">
          <TelemetryChart data={data} />
        </div>
      </div>
      <div className="flex items-end justify-between border-t border-hairline px-6 py-6">
        <div>
          <p className="text-[12px] text-muted-copy">Session traffic</p>
          <p className="type-display mt-2 text-[27px] leading-none tracking-[-0.02em] text-primary">
            1.84 <span className="font-sans text-[12px] font-medium tracking-normal text-body">GB</span>
          </p>
        </div>
        <Activity aria-hidden="true" className="size-5 text-lavender" strokeWidth={1.7} />
      </div>
    </Card>
  )
}
