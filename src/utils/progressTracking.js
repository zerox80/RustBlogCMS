// Storage configuration constants
const STORAGE_KEY = 'rust_blog_cms_progress_v2';
const BOOKMARKS_KEY = 'rust_blog_cms_bookmarks_v2';
const STORAGE_PREFIX = 'rust_blog_cms_';
const MAX_STORAGE_RETRIES = 2;
// Performance and security limits
const MAX_STORAGE_SIZE = 5 * 1024 * 1024; // 5MB limit
const MAX_TUTORIALS_PER_USER = 1000;
// In-memory storage fallback
let memoryStorage = {};
let isStorageAvailable = null;
const isLocalStorageAvailable = () => {
  if (isStorageAvailable !== null) {
    return isStorageAvailable;
  }
  try {
    const test = '__localStorage_test__';
    localStorage.setItem(test, test);
    localStorage.removeItem(test);
    isStorageAvailable = true;
  } catch (error) {
    isStorageAvailable = false;
    console.warn('localStorage not available, using memory fallback:', error.message);
  }
  return isStorageAvailable;
};
const safeGetStorage = (key, defaultValue = null) => {
  try {
    if (isLocalStorageAvailable()) {
      const stored = localStorage.getItem(key);
      return stored ? JSON.parse(stored) : defaultValue;
    }
    return memoryStorage[key] || defaultValue;
  } catch (error) {
    console.error(`Failed to retrieve ${key} from storage:`, error);
    return defaultValue;
  }
};
const safeSetStorage = (key, value, attempt = 0) => {
  try {
    const serialized = JSON.stringify(value);
    // Check size limits
    if (serialized.length > MAX_STORAGE_SIZE) {
      console.warn(`Data size (${serialized.length} bytes) exceeds maximum allowed size`);
      return false;
    }
    if (isLocalStorageAvailable()) {
      localStorage.setItem(key, serialized);
    } else {
      memoryStorage[key] = value;
    }
    return true;
  } catch (error) {
    if (error.name === 'QuotaExceededError') {
      console.warn('Storage quota exceeded, attempting to free space...');
      const cleaned = attemptStorageCleanup();
      if (!cleaned || attempt >= MAX_STORAGE_RETRIES) {
        console.error('Storage cleanup did not free enough space. Aborting write.');
        return false;
      }
      return safeSetStorage(key, value, attempt + 1);
    }
    console.error(`Failed to store ${key} to storage:`, error);
    return false;
  }
};
const attemptStorageCleanup = () => {
  if (!isLocalStorageAvailable()) {
    return false;
  }
  try {
    const entries = Object.keys(localStorage)
      .filter((key) => key.startsWith(STORAGE_PREFIX))
      .map((key) => ({ key, size: (localStorage.getItem(key) || '').length }))
      .sort((a, b) => b.size - a.size);

    let totalSize = entries.reduce((size, entry) => size + entry.size, 0);
    if (totalSize <= MAX_STORAGE_SIZE * 0.9) {
      return false;
    }

    console.warn('Cleaning up stored progress data to free space');
    let removed = false;
    for (const entry of entries) {
      localStorage.removeItem(entry.key);
      totalSize -= entry.size;
      removed = true;
      if (totalSize <= MAX_STORAGE_SIZE * 0.8) {
        break;
      }
    }

    return removed;
  } catch (error) {
    console.error('Storage cleanup failed:', error);
    return false;
  }
};
const validateTutorialId = (tutorialId) => {
  if (typeof tutorialId !== 'string') {
    return false;
  }
  // Check length and format
  return tutorialId.length > 0 &&
    tutorialId.length <= 100 &&
    /^[a-zA-Z0-9_-]+$/.test(tutorialId);
};
export const getProgress = () => {
  try {
    return safeGetStorage(STORAGE_KEY, {});
  } catch (error) {
    console.error('Failed to load progress:', error);
    return {};
  }
};
export const markAsRead = (tutorialId, options = {}) => {
  if (!validateTutorialId(tutorialId)) {
    throw new TypeError(`Invalid tutorial ID: ${tutorialId}`);
  }
  try {
    const progress = getProgress();
    // Enforce quota limits
    if (Object.keys(progress).length >= MAX_TUTORIALS_PER_USER) {
      console.warn('Maximum number of tracked tutorials reached');
      return false;
    }
    // Create comprehensive progress entry
    progress[tutorialId] = {
      read: true,
      timestamp: new Date().toISOString(),
      completionTime: options.completionTime || null,
      score: options.score || null,
      completedSections: Array.isArray(options.completedSections)
        ? options.completedSections
        : [],
      version: '2.0.0'
    };
    return safeSetStorage(STORAGE_KEY, progress);
  } catch (error) {
    console.error('Failed to save progress:', error);
    return false;
  }
};
export const isRead = (tutorialId) => {
  if (!validateTutorialId(tutorialId)) {
    return false;
  }
  try {
    const progress = getProgress();
    return progress[tutorialId]?.read === true;
  } catch (error) {
    console.error('Failed to check read status:', error);
    return false;
  }
};
export const clearProgress = (confirm = false) => {
  if (!confirm) {
    throw new Error('clearProgress requires explicit confirmation. Pass true to proceed.');
  }
  try {
    if (isLocalStorageAvailable()) {
      localStorage.removeItem(STORAGE_KEY);
    } else {
      delete memoryStorage[STORAGE_KEY];
    }
    console.info('All tutorial progress has been cleared');
    return true;
  } catch (error) {
    console.error('Failed to clear progress:', error);
    return false;
  }
};
export const getBookmarks = () => {
  try {
    const bookmarks = safeGetStorage(BOOKMARKS_KEY, []);
    // Validate and deduplicate
    if (Array.isArray(bookmarks)) {
      return [...new Set(bookmarks)].filter(id => validateTutorialId(id));
    }
    return [];
  } catch (error) {
    console.error('Failed to load bookmarks:', error);
    return [];
  }
};
export const toggleBookmark = (tutorialId) => {
  if (!validateTutorialId(tutorialId)) {
    throw new TypeError(`Invalid tutorial ID: ${tutorialId}`);
  }
  try {
    const bookmarks = getBookmarks();
    const index = bookmarks.indexOf(tutorialId);
    if (index > -1) {
      // Remove bookmark
      bookmarks.splice(index, 1);
      console.info(`Removed bookmark for tutorial: ${tutorialId}`);
    } else {
      // Add bookmark with quota check
      if (bookmarks.length >= MAX_TUTORIALS_PER_USER) {
        throw new Error(`Maximum bookmark limit (${MAX_TUTORIALS_PER_USER}) reached`);
      }
      bookmarks.push(tutorialId);
      console.info(`Added bookmark for tutorial: ${tutorialId}`);
    }
    const success = safeSetStorage(BOOKMARKS_KEY, bookmarks);
    if (!success) {
      throw new Error('Failed to save bookmarks due to storage error');
    }
    return bookmarks;
  } catch (error) {
    console.error('Failed to toggle bookmark:', error);
    return getBookmarks();
  }
};
export const isBookmarked = (tutorialId) => {
  if (!validateTutorialId(tutorialId)) {
    return false;
  }
  try {
    const bookmarks = getBookmarks();
    return bookmarks.includes(tutorialId);
  } catch (error) {
    console.error('Failed to check bookmark status:', error);
    return false;
  }
};
export const clearBookmarks = (confirm = false) => {
  if (!confirm) {
    throw new Error('clearBookmarks requires explicit confirmation. Pass true to proceed.');
  }
  try {
    if (isLocalStorageAvailable()) {
      localStorage.removeItem(BOOKMARKS_KEY);
    } else {
      delete memoryStorage[BOOKMARKS_KEY];
    }
    console.info('All bookmarks have been cleared');
    return true;
  } catch (error) {
    console.error('Failed to clear bookmarks:', error);
    return false;
  }
};
export const getProgressStatistics = () => {
  try {
    const progress = getProgress();
    const bookmarks = getBookmarks();
    const tutorialIds = Object.keys(progress);
    const completedIds = tutorialIds.filter(id => progress[id].read);
    // Calculate completion rate
    const completionRate = tutorialIds.length > 0
      ? Math.round((completedIds.length / tutorialIds.length) * 100)
      : 0;
    // Find last activity timestamp
    const timestamps = completedIds
      .map(id => progress[id].timestamp)
      .filter(Boolean)
      .sort((a, b) => new Date(b) - new Date(a));
    const lastActivity = timestamps.length > 0 ? new Date(timestamps[0]) : null;
    // Get recently completed tutorials (last 5)
    const recentlyRead = completedIds
      .sort((a, b) => new Date(progress[b].timestamp) - new Date(progress[a].timestamp))
      .slice(0, 5);
    return {
      totalTutorials: tutorialIds.length,
      completedTutorials: completedIds.length,
      totalBookmarks: bookmarks.length,
      completionRate,
      lastActivity,
      recentlyRead
    };
  } catch (error) {
    console.error('Failed to calculate progress statistics:', error);
    return {
      totalTutorials: 0,
      completedTutorials: 0,
      totalBookmarks: 0,
      completionRate: 0,
      lastActivity: null,
      recentlyRead: []
    };
  }
};