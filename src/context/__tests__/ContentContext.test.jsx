import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { ContentProvider, useContent, DEFAULT_CONTENT } from '../ContentContext';
import { api } from '../../api/client';

// Mock the api client
vi.mock('../../api/client', () => ({
  api: {
    getSiteContent: vi.fn(),
    getNavigation: vi.fn(),
    listPublishedPages: vi.fn(),
    updateSiteContentSection: vi.fn(),
    getPublishedPage: vi.fn(),
    getPublishedPost: vi.fn(),
  },
}));

describe('ContentContext', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.resetAllMocks();
  });

  it('initializes by loading content and navigation', async () => {
    const mockContent = {
      items: [
        { section: 'hero', content: { title: 'New Hero Title' } }
      ]
    };
    const mockNav = { items: [{ slug: 'page1', label: 'Page 1' }] };
    const mockPages = ['page1'];

    api.getSiteContent.mockResolvedValue(mockContent);
    api.getNavigation.mockResolvedValue(mockNav);
    api.listPublishedPages.mockResolvedValue(mockPages);

    const { result } = renderHook(() => useContent(), {
      wrapper: ContentProvider,
    });

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
      expect(result.current.navigation.loading).toBe(false);
    });

    // Check if content was merged correctly
    expect(result.current.content.hero.title).toBe('New Hero Title');
    // Check if other sections remain default
    expect(result.current.content.site_meta).toBeDefined();

    expect(api.getSiteContent).toHaveBeenCalled();
    expect(api.getNavigation).toHaveBeenCalled();
    expect(api.listPublishedPages).toHaveBeenCalled();
  });

  it('handles content load failure by keeping defaults', async () => {
    api.getSiteContent.mockRejectedValue(new Error('Failed to load'));
    api.getNavigation.mockResolvedValue({ items: [] });
    api.listPublishedPages.mockResolvedValue([]);

    const { result } = renderHook(() => useContent(), {
      wrapper: ContentProvider,
    });

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.error).toBeDefined();
    expect(result.current.content).toEqual(DEFAULT_CONTENT);
  });

  it('updateSection updates content and calls api', async () => {
    // Initial load
    api.getSiteContent.mockResolvedValue({ items: [] });
    api.getNavigation.mockResolvedValue({ items: [] });
    api.listPublishedPages.mockResolvedValue([]);

    const { result } = renderHook(() => useContent(), {
      wrapper: ContentProvider,
    });

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    const newHeroContent = { ...DEFAULT_CONTENT.hero, title: { line1: 'Updated' } };
    api.updateSiteContentSection.mockResolvedValue({ content: newHeroContent });

    await act(async () => {
      await result.current.updateSection('hero', newHeroContent);
    });

    expect(api.updateSiteContentSection).toHaveBeenCalledWith('hero', newHeroContent);
    expect(result.current.content.hero).toEqual(newHeroContent);
  });

  it('fetches published page and caches it', async () => {
    // Initial load setup
    api.getSiteContent.mockResolvedValue({ items: [] });
    api.getNavigation.mockResolvedValue({ items: [] });
    api.listPublishedPages.mockResolvedValue(['about-us']);

    const { result } = renderHook(() => useContent(), {
      wrapper: ContentProvider,
    });

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    const mockPageData = { title: 'About Us', content: 'Some content' };
    api.getPublishedPage.mockResolvedValue(mockPageData);

    let pageData;
    await act(async () => {
      pageData = await result.current.pages.fetch('about-us');
    });

    expect(pageData).toEqual(mockPageData);
    expect(api.getPublishedPage).toHaveBeenCalledWith('about-us', expect.anything());

    // Check cache
    expect(result.current.pages.cache['about-us']).toEqual(mockPageData);

    // Call again, should not call API (cache hit)
    vi.clearAllMocks(); // Clear API mock calls

    await act(async () => {
      pageData = await result.current.pages.fetch('about-us');
    });

    expect(pageData).toEqual(mockPageData);
    expect(api.getPublishedPage).not.toHaveBeenCalled();
  });
});
