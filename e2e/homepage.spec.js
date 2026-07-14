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
  '/api/public/pages/e2e-page/posts/e2e-article': {
    post: {
      id: 'e2e-post',
      title: 'E2E article',
      slug: 'e2e-article',
      excerpt: 'Test content',
      content_markdown: '# Article heading\n\nReadable body text with **strong emphasis**.',
      published_at: '2026-07-14T08:00:00Z',
    },
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

  test('uses the editorial light theme', async ({ page }) => {
    await page.goto('/')

    await expect(page.locator('html')).not.toHaveClass(/dark/)
    await expect(page.locator('main')).toHaveCSS('background-color', 'rgb(244, 241, 234)')
  })

  test('keeps article text dark when the operating system prefers dark mode', async ({ page }) => {
    await page.emulateMedia({ colorScheme: 'dark' })
    await page.goto('/posts/e2e-page/e2e-article')

    await expect(page.locator('article header h1')).toHaveCSS('color', 'rgb(23, 23, 19)')
    await expect(page.locator('.editorial-markdown p')).toHaveCSS('color', 'rgb(52, 52, 45)')
    await expect(page.locator('.editorial-markdown strong')).toHaveCSS('color', 'rgb(23, 23, 19)')
  })

  test('shows posts returned through published page slugs', async ({ page }) => {
    await page.goto('/')

    await expect(page.getByText('E2E article')).toBeVisible()
    await expect(page.getByRole('link', { name: 'Beitrag lesen: E2E article' })).toHaveAttribute(
      'href',
      '/posts/e2e-page/e2e-article',
    )
    await expect(page.locator('nav a[href^="/pages/"]')).toHaveCount(0)
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
