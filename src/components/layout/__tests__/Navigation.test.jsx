import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import Header from '../Header'
import Footer from '../Footer'

const authState = vi.hoisted(() => ({ authenticated: false }))
const navigation = vi.hoisted(() => ({
  static: [
    { id: 'stories', label: 'Stories', type: 'section', value: 'stories' },
    { id: 'topics', label: 'Topics', type: 'section', value: 'topics' },
    { id: 'about', label: 'About', type: 'section', value: 'about' },
    { id: 'legacy-page', label: 'Legacy page', type: 'route', path: '/pages/legacy' },
  ],
  dynamic: [
    { id: 'page-one', label: 'Page one', path: '/pages/one' },
    { id: 'page-two', label: 'Page two', path: '/pages/two' },
    { id: 'page-three', label: 'Page three', path: '/pages/three' },
  ],
}))

const sections = vi.hoisted(() => ({
  header: { brand: { name: 'CMS brand' }, cta: { guestLabel: 'Sign in', authLabel: 'Dashboard' } },
  footer: {
    brand: { title: 'CMS brand', description: 'CMS description' },
    quickLinks: [
      { label: 'Latest', target: { type: 'section', value: 'stories' } },
      { label: 'CMS page', target: { type: 'page', value: 'custom' } },
    ],
    contactLinks: [],
    bottom: { copyright: '© {year} CMS', signature: 'Signature' },
  },
}))

vi.mock('../../../context/ContentContext', () => ({
  useContent: () => ({ navigation, getSection: (section) => sections[section] }),
}))

vi.mock('../../../context/EditContext', () => ({
  useEdit: () => ({ isEditing: false, toggleEditMode: vi.fn() }),
}))

vi.mock('../../../context/AuthContext', () => ({
  useAuth: () => ({ isAuthenticated: authState.authenticated }),
}))

vi.mock('../../cms/EditableText', () => ({
  default: ({ value }) => <>{value}</>,
}))

const renderInRouter = (component) => render(<MemoryRouter>{component}</MemoryRouter>)

describe('CMS navigation', () => {
  beforeEach(() => {
    authState.authenticated = false
  })

  it('shows one-page anchors, hides legacy page routes, and includes login', () => {
    renderInRouter(<Header />)

    expect(screen.getByRole('link', { name: 'Stories' })).toHaveAttribute('href', '#stories')
    expect(screen.queryByRole('link', { name: 'Topics' })).not.toBeInTheDocument()
    expect(screen.queryByRole('link', { name: 'Legacy page' })).not.toBeInTheDocument()
    expect(screen.queryByRole('link', { name: 'Page three' })).not.toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Sign in' })).toHaveAttribute('href', '/login')
  })

  it('links authenticated users to the admin area', () => {
    authState.authenticated = true
    renderInRouter(<Header />)

    expect(screen.getByRole('link', { name: 'Dashboard' })).toHaveAttribute('href', '/admin')
  })

  it('uses one-page quick links without exposing CMS page navigation', () => {
    renderInRouter(<Footer />)

    expect(screen.getByRole('link', { name: 'Latest' })).toHaveAttribute('href', '/#stories')
    expect(screen.queryByRole('link', { name: 'CMS page' })).not.toBeInTheDocument()
    expect(screen.queryByRole('link', { name: 'Page three' })).not.toBeInTheDocument()
  })
})
