/**
 * Main Entry Point for Linux Tutorial CMS Application
 * 
 * This file serves as the bootstrap for the entire React application.
 * It initializes the React root, applies global styles, and renders the main App component.
 * 
 * Key Responsibilities:
 * - Create React root DOM node
 * - Apply global CSS styles and syntax highlighting themes
 * - Wrap application in React.StrictMode for development checks
 * - Mount the application to the DOM
 * 
 * @module main
 */

// Import React library - core library for building user interfaces
import React from 'react'

// Import ReactDOM client API - provides DOM-specific methods for React 18+
import ReactDOM from 'react-dom/client'

// Import root App component - main application component with routing and providers
import App from './App.jsx'

// Import global CSS styles - Tailwind CSS and custom application styles
import './index.css'
import './i18n/config'

// Import syntax highlighting theme - GitHub Dark theme for code blocks
import 'highlight.js/styles/github-dark.css'

/**
 * Create React root and render the application
 * 
 * Uses React 18's createRoot API for concurrent rendering features:
 * - Automatic batching of state updates
 * - Transitions for better UX
 * - Suspense for data fetching
 * 
 * React.StrictMode wrapper provides:
 * - Detection of unsafe lifecycle methods
 * - Warning about legacy string ref API usage
 * - Warning about deprecated findDOMNode usage
 * - Detection of unexpected side effects
 * - Detection of legacy context API
 * 
 * @see {@link https://react.dev/reference/react-dom/client/createRoot} for createRoot documentation
 * @see {@link https://react.dev/reference/react/StrictMode} for StrictMode documentation
 */
ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
