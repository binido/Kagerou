import { useEffect, useState, type FormEvent } from 'react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Field, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { kagerouApi } from '@/lib/tauri-api'
import { routeOutboundOptions } from '@/types/kagerou'
import type { MatchAnalysis, MatchKind, MatchWarning, Outbound, RoutingRule } from '@/types/kagerou'

type OutboundTranslationKey = 'table.direct' | 'table.proxy' | 'table.block'

const outboundKeys: Record<Outbound, OutboundTranslationKey> = {
  Direct: 'table.direct',
  Proxy: 'table.proxy',
  Block: 'table.block',
}

const kindKeys = {
  domain: 'dialog.kindDomain',
  'domain-suffix': 'dialog.kindDomainSuffix',
  'ip-cidr': 'dialog.kindIpCidr',
} as const satisfies Record<MatchKind, string>

const warningKeys = {
  wildcard: 'dialog.warningWildcard',
  url: 'dialog.warningUrl',
  'non-ascii': 'dialog.warningNonAscii',
  'invalid-prefix': 'dialog.warningInvalidPrefix',
  'invalid-domain': 'dialog.warningInvalidDomain',
} as const satisfies Record<MatchWarning, string>

interface EditRuleDialogProps {
  /** An existing rule to edit, `'new'` to add one, `null` when closed. */
  rule?: RoutingRule | 'new' | null
  onOpenChange: (open: boolean) => void
  onSave: (patch: { match: string; outbound: Outbound }) => void
}

export function EditRuleDialog({ rule, onOpenChange, onSave }: EditRuleDialogProps) {
  const { t } = useTranslation('routing')
  const adding = rule === 'new'
  const existing = adding ? null : rule
  const [match, setMatch] = useState(existing?.match ?? '')
  // Deliberately empty when adding: defaulting to Direct means a stray click
  // on Save sends traffic around the tunnel, which is the mistake that looks
  // like it worked.
  const [outbound, setOutbound] = useState<Outbound | ''>(existing?.outbound ?? '')
  const [error, setError] = useState('')
  const [analysis, setAnalysis] = useState<MatchAnalysis | null>(null)

  // Debounced, so the hint does not shout about a half-typed domain. The
  // previous answer stays on screen until the next one lands, which is the
  // same 200ms of staleness the debounce is already asking for.
  useEffect(() => {
    const value = match.trim()
    if (!value) return
    const timer = setTimeout(() => {
      kagerouApi.analyzeRuleMatch(value).then(setAnalysis).catch(() => undefined)
    }, 200)
    return () => clearTimeout(timer)
  }, [match])

  const hint = match.trim() ? analysis : null

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (!match.trim()) {
      setError(t('dialog.error'))
      return
    }
    if (!outbound) {
      setError(t('dialog.outboundError'))
      return
    }
    // The stored value is what the backend classifier normalized, so the rule
    // in the config is the one the hint described.
    onSave({ match: hint?.normalized ?? match.trim(), outbound })
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={Boolean(rule)}>
      <DialogContent className="border-hairline bg-raised text-primary sm:max-w-[420px]">
        <DialogHeader><DialogTitle className="type-display text-2xl text-primary">{adding ? t('dialog.addTitle') : t('dialog.title')}</DialogTitle><DialogDescription className="text-[12px] text-muted-copy">{t('dialog.description')}</DialogDescription></DialogHeader>
        <form className="space-y-5" onSubmit={submit}>
          <Field>
            <FieldLabel className="text-[12px] text-primary" htmlFor="rule-match">{t('dialog.match')}</FieldLabel>
            <Input aria-describedby="rule-match-hint" className="h-10 border-hairline bg-surface text-[13px]" id="rule-match" onChange={(event) => setMatch(event.target.value)} placeholder={t('dialog.matchPlaceholder')} value={match} />
            <p className="min-h-[15px] text-[11px] leading-4" id="rule-match-hint">
              {hint ? <span className={hint.warning ? 'text-warn' : 'text-quiet'}>{hint.warning ? t(warningKeys[hint.warning]) : t(kindKeys[hint.kind])}</span> : null}
            </p>
          </Field>
          <Field><FieldLabel className="text-[12px] text-primary" htmlFor="rule-outbound">{t('dialog.outbound')}</FieldLabel><Select onValueChange={(value) => setOutbound(value as Outbound)} value={outbound}><SelectTrigger className="w-full border-hairline bg-surface text-[13px]" id="rule-outbound"><SelectValue placeholder={t('dialog.outboundPlaceholder')} /></SelectTrigger><SelectContent className="border-hairline bg-raised text-body">{routeOutboundOptions.map((option) => <SelectItem key={option} value={option}>{t(outboundKeys[option])}</SelectItem>)}</SelectContent></Select></Field>
          {error ? <p className="text-[11px] text-bad">{error}</p> : null}
          <DialogFooter><Button onClick={() => onOpenChange(false)} type="button" variant="ghost">{t('dialog.cancel')}</Button><Button className="bg-lavender text-ink hover:bg-lavender-hi" type="submit">{adding ? t('dialog.add') : t('dialog.save')}</Button></DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
