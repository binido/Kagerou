import { useState, type FormEvent } from 'react'

import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Field, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { Outbound, RoutingRule } from '@/types/kagerou'
import { routeOutboundOptions } from '@/lib/mock-data'

interface EditRuleDialogProps {
  rule?: RoutingRule | null
  onOpenChange: (open: boolean) => void
  onSave: (patch: Pick<RoutingRule, 'match' | 'outbound'>) => void
}

export function EditRuleDialog({ rule, onOpenChange, onSave }: EditRuleDialogProps) {
  const [match, setMatch] = useState(rule?.match ?? '')
  const [outbound, setOutbound] = useState<Outbound>(rule?.outbound ?? 'Direct')
  const [error, setError] = useState('')

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (!match.trim()) {
      setError('Enter a domain, IP range, or hostname.')
      return
    }
    onSave({ match: match.trim(), outbound })
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={Boolean(rule)}>
      <DialogContent className="border-[#3b3a45] bg-raised text-primary sm:max-w-[420px]">
        <DialogHeader><DialogTitle className="type-display text-2xl text-primary">Edit routing rule</DialogTitle><DialogDescription className="text-[12px] text-muted-copy">Rules are evaluated from top to bottom. The first match wins.</DialogDescription></DialogHeader>
        <form className="space-y-5" onSubmit={submit}>
          <Field><FieldLabel className="text-[12px] text-primary" htmlFor="rule-match">Match</FieldLabel><Input className="h-10 border-white/10 bg-surface text-[13px]" id="rule-match" onChange={(event) => setMatch(event.target.value)} value={match} /></Field>
          <Field><FieldLabel className="text-[12px] text-primary" htmlFor="rule-outbound">Outbound</FieldLabel><Select onValueChange={(value) => setOutbound(value as Outbound)} value={outbound}><SelectTrigger className="w-full border-white/10 bg-surface text-[13px]" id="rule-outbound"><SelectValue /></SelectTrigger><SelectContent className="border-hairline bg-raised text-body">{routeOutboundOptions.map((option) => <SelectItem key={option} value={option}>{option}</SelectItem>)}</SelectContent></Select></Field>
          {error ? <p className="text-[11px] text-bad">{error}</p> : null}
          <DialogFooter><Button onClick={() => onOpenChange(false)} type="button" variant="ghost">Cancel</Button><Button className="bg-lavender text-ink hover:bg-lavender-hi" type="submit">Save rule</Button></DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
