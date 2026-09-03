import type { ReactNode } from 'react'

export function SettingsSection({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="mt-9 first:mt-0" aria-labelledby={`${title.toLowerCase().replace(/\s+/g, '-')}-heading`}>
      <h2 className="type-eyebrow mb-3" id={`${title.toLowerCase().replace(/\s+/g, '-')}-heading`}>{title}</h2>
      <div>{children}</div>
    </section>
  )
}
