import { useState, type FormEvent } from 'react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Field, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { routeOutboundOptions } from '@/lib/mock-data'
import type { Outbound, RoutingRule } from '@/types/kagerou'

type OutboundTranslationKey = 'table.direct' | 'table.proxy' | 'table.block'

const outboundKeys: Record<Outbound, OutboundTranslationKey> = {
  Direct: 'table.direct',
  Proxy: 'table.proxy',
  Block: 'table.block',
}

interface EditRuleDialogProps {
  rule?: RoutingRule | null
  onOpenChange: (open: boolean) => void
  onSave: (patch: Pick<RoutingRule, 'match' | 'outbound'>) => void
}

export function EditRuleDialog({ rule, onOpenChange, onSave }: EditRuleDialogProps) {
  const { t } = useTranslation('routing')
  const [match, setMatch] = useState(rule?.match ?? '')
  const [outbound, setOutbound] = useState<Outbound>(rule?.outbound ?? 'Direct')
  const [error, setError] = useState('')

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (!match.trim()) {
      setError(t('dialog.error'))
      return
    }
    onSave({ match: match.trim(), outbound })
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={Boolean(rule)}>
      <DialogContent className="border-hairline bg-raised text-primary sm:max-w-[420px]">
        <DialogHeader><DialogTitle className="type-display text-2xl text-primary">{t('dialog.title')}</DialogTitle><DialogDescription className="text-[12px] text-muted-copy">{t('dialog.description')}</DialogDescription></DialogHeader>
        <form className="space-y-5" onSubmit={submit}>
          <Field><FieldLabel className="text-[12px] text-primary" htmlFor="rule-match">{t('dialog.match')}</FieldLabel><Input className="h-10 border-hairline bg-surface text-[13px]" id="rule-match" onChange={(event) => setMatch(event.target.value)} placeholder={t('dialog.matchPlaceholder')} value={match} /></Field>
          <Field><FieldLabel className="text-[12px] text-primary" htmlFor="rule-outbound">{t('dialog.outbound')}</FieldLabel><Select onValueChange={(value) => setOutbound(value as Outbound)} value={outbound}><SelectTrigger className="w-full border-hairline bg-surface text-[13px]" id="rule-outbound"><SelectValue /></SelectTrigger><SelectContent className="border-hairline bg-raised text-body">{routeOutboundOptions.map((option) => <SelectItem key={option} value={option}>{t(outboundKeys[option])}</SelectItem>)}</SelectContent></Select></Field>
          {error ? <p className="text-[11px] text-bad">{error}</p> : null}
          <DialogFooter><Button onClick={() => onOpenChange(false)} type="button" variant="ghost">{t('dialog.cancel')}</Button><Button className="bg-lavender text-ink hover:bg-lavender-hi" type="submit">{t('dialog.save')}</Button></DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
