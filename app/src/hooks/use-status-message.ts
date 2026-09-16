import { useState } from 'react'

export type MessageTone = 'muted' | 'good' | 'bad'

export interface StatusMessage {
  text: string
  tone: MessageTone
}

/** The one live region a page keeps for saying what just happened. Separate
 * from toasts: this is the running commentary a screen reader follows. */
export function useStatusMessage() {
  const [message, setMessage] = useState<StatusMessage>({ text: '', tone: 'muted' })

  const say = (text: string, tone: MessageTone = 'muted') => setMessage({ text, tone })

  return { message, say }
}
