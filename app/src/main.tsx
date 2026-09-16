import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import './i18n'
import App from './App'
import { initializeTheme } from '@/themes/runtime'

initializeTheme()

// The WebView's own menu - reload, back, inspect - belongs to a browser, not
// to this window. Inputs keep theirs: right-click paste is how a subscription
// link usually arrives. The DEV guard keeps the inspector reachable.
if (!import.meta.env.DEV) {
  document.addEventListener('contextmenu', (event) => {
    if (event.target instanceof HTMLElement && event.target.closest('input, textarea')) return
    event.preventDefault()
  })
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
