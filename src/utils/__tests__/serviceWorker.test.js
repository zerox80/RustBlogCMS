import { readFileSync } from 'node:fs'
import vm from 'node:vm'
import { describe, expect, it, vi } from 'vitest'

const workerPath = 'public/sw.js'

const loadWorker = () => {
  const listeners = new Map()
  const self = {
    location: { origin: 'https://cms.example.test' },
    addEventListener: (type, handler) => listeners.set(type, handler),
  }
  const context = vm.createContext({
    self,
    caches: {},
    console: { log: vi.fn(), error: vi.fn() },
    URL,
  })

  new vm.Script(readFileSync(workerPath, 'utf8')).runInContext(context)
  return listeners
}

describe('service worker cache policy', () => {
  it('does not intercept API requests that may contain private data', () => {
    const listeners = loadWorker()
    const event = {
      request: {
        url: 'https://cms.example.test/api/auth/me',
        method: 'GET',
      },
      respondWith: vi.fn(),
    }

    listeners.get('fetch')(event)

    expect(event.respondWith).not.toHaveBeenCalled()
  })

  it('does not intercept non-GET requests', () => {
    const listeners = loadWorker()
    const event = {
      request: {
        url: 'https://cms.example.test/uploads/image.png',
        method: 'POST',
      },
      respondWith: vi.fn(),
    }

    listeners.get('fetch')(event)

    expect(event.respondWith).not.toHaveBeenCalled()
  })
})
