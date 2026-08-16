import { useEffect, useState } from 'react'

type ApiStatus = 'checking' | 'ok' | 'unreachable'

function App() {
  const [apiStatus, setApiStatus] = useState<ApiStatus>('checking')

  useEffect(() => {
    fetch('/healthz')
      .then((res) => setApiStatus(res.ok ? 'ok' : 'unreachable'))
      .catch(() => setApiStatus('unreachable'))
  }, [])

  return (
    <div className="flex min-h-screen items-center justify-center bg-neutral-950 text-neutral-100">
      <div className="text-center">
        <h1 className="text-2xl font-semibold">Circus</h1>
        <p className="mt-2 text-sm text-neutral-400">
          backend:{' '}
          <span
            className={
              apiStatus === 'ok'
                ? 'text-emerald-400'
                : apiStatus === 'checking'
                  ? 'text-neutral-400'
                  : 'text-red-400'
            }
          >
            {apiStatus}
          </span>
        </p>
      </div>
    </div>
  )
}

export default App
