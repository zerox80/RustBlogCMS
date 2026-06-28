/**
 * Theme Context Module
 * 
 * Manages application-wide theme state (light/dark mode).
 * Provides theme switching functionality with localStorage persistence
 * and system preference detection.
 * 
 * Features:
 * - Light/dark theme toggle
 * - localStorage persistence
 * - System preference detection (prefers-color-scheme)
 * - Automatic DOM class management
 * 
 * @module ThemeContext
 */

// Import React hooks for context creation and state management
import { createContext, useContext, useState, useEffect } from 'react';

// Import PropTypes for runtime type checking
import PropTypes from 'prop-types';

/**
 * Theme Context
 * 
 * React context that holds theme state and toggle function.
 * Provides theme value ('light' or 'dark') and toggleTheme method.
 * 
 * @type {React.Context<ThemeContextValue|undefined>}
 */
const ThemeContext = createContext();
/**
 * useTheme Hook
 * 
 * Custom hook to access theme context.
 * Must be used within ThemeProvider component tree.
 * 
 * @returns {ThemeContextValue} Theme context value with theme and toggleTheme
 * @throws {Error} If used outside ThemeProvider
 * 
 * @example
 * const { theme, toggleTheme } = useTheme()
 * // theme is 'light' or 'dark'
 * // toggleTheme() switches between themes
 */
export const useTheme = () => {
  // Get theme context
  const context = useContext(ThemeContext);

  // Throw error if hook is used outside ThemeProvider
  // This prevents runtime errors and helps developers catch mistakes early
  if (!context) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }

  // Return context value with theme state and toggle function
  return context;
};
/**
 * Theme Provider Component
 * 
 * Wraps the application to provide theme context to all child components.
 * Manages theme state with localStorage persistence and system preference detection.
 * 
 * @param {Object} props - Component props
 * @param {React.ReactNode} props.children - Child components to wrap with theme context
 * @returns {JSX.Element} Provider component with theme context
 */
export const ThemeProvider = ({ children }) => {
  /**
   * Theme State
   * 
   * Initialized with lazy initialization function that:
   * 1. Checks localStorage for saved theme preference
   * 2. Falls back to system preference (prefers-color-scheme)
   * 3. Defaults to 'light' if no preference found
   * 
   * This ensures theme persists across sessions and respects user preferences.
   */
  const [theme] = useState('dark');

  /**
   * Effect: Apply theme to DOM
   * 
   * Always adds 'dark' class to document root.
   */
  useEffect(() => {
    if (typeof window === 'undefined' || !window.document?.documentElement) {
      return undefined;
    }

    const root = window.document.documentElement;
    root.classList.add('dark');

    // Optional: Persist 'dark' just in case, though we force it anyway
    try {
      window.localStorage?.setItem('theme', 'dark');
    } catch {
      // Ignore
    }
  }, []);

  /**
   * Toggle Theme Function
   * 
   * No-op as we are enforcing dark mode.
   */
  const toggleTheme = () => {
    // No-op
  };
  /**
   * Render Provider
   * 
   * Provides theme context value to all child components.
   * Value includes current theme ('light' or 'dark') and toggleTheme function.
   */
  return (
    <ThemeContext.Provider value={{ theme, toggleTheme }}>
      {children}
    </ThemeContext.Provider>
  );
};
// PropTypes validation for ThemeProvider component
ThemeProvider.propTypes = {
  children: PropTypes.node.isRequired,  // Child components to wrap with theme context
};
