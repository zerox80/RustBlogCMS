import { test, expect } from '@playwright/test'

const API_RESPONSES = {
  '/api/auth/me': { username: 'e2e-admin', role: 'admin' },
  '/api/content': { items: [] },
  '/api/public/navigation': { items: [] },
  '/api/public/published-pages': ['e2e-page'],
  '/api/public/pages/e2e-page': {
    page: { title: 'E2E Page' },
    posts: [{ id: 'e2e-post', title: 'E2E article', slug: 'e2e-article', excerpt: 'Test content' }],
  },
  '/api/tutorials': [],
}

test.describe('Homepage', () => {
  test.beforeEach(async ({ page }) => {
    await page.route('**/*', async (route) => {
      const path = new URL(route.request().url()).pathname
      if (!path.startsWith('/api/')) {
        await route.continue()
        return
      }
      await route.fulfill({
        contentType: 'application/json',
        body: JSON.stringify(API_RESPONSES[path] ?? []),
      })
    })
  })

  test('loads the primary page structure', async ({ page }) => {
    await page.goto('/')

    await expect(page.locator('h1')).toBeVisible()
    await expect(page.locator('nav')).toBeVisible()
  })

  test('enforces dark mode', async ({ page }) => {
    await page.goto('/')

    await expect(page.locator('html')).toHaveClass(/dark/)
  })

  test('shows posts returned through published page slugs', async ({ page }) => {
    await page.goto('/')

    await expect(page.getByText('E2E article')).toBeVisible()
  })

  test('exposes an accessible mobile menu', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 })
    await page.goto('/')

    const menuButton = page.getByRole('button', { name: /menü öffnen/i })
    await expect(menuButton).toBeVisible()
    await menuButton.click()
    await expect(page.locator('#mobile-navigation')).toBeVisible()
  })
})
