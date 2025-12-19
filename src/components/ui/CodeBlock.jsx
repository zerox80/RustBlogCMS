import { useState } from 'react';
import { Check, Copy } from 'lucide-react';
import PropTypes from 'prop-types';
/**
 * A reusable code snippet component with a "Copy to Clipboard" feature.
 * 
 * Includes:
 * - Syntax highlighting (via parent CSS/Markdown styles).
 * - Automatic text extraction from children (handles strings and React nodes).
 * - Animated copy-success feedback.
 */
const CodeBlock = ({ children, className }) => {
  const [copied, setCopied] = useState(false);
  const handleCopy = async () => {
    // Extract text content from children, handling both direct strings and React elements
    const code = children?.props?.children || children;
    // Ensure we're working with a string and trim whitespace
    const textToCopy = typeof code === 'string' ? code : String(code);
    try {
      // Use the modern Clipboard API for secure copying
      await navigator.clipboard.writeText(textToCopy.trim());
      // Update state to show success feedback
      setCopied(true);
      // Reset feedback after 2 seconds
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      // Handle clipboard errors (permissions, browser support, etc.)
      console.error('Failed to copy code:', err);
      // Note: We don't show error feedback to user to avoid confusion
    }
  };
  return (
    <div className="relative group">
      { }
      <button
        onClick={handleCopy}
        className="absolute right-2 top-2 p-2 rounded-lg bg-gray-700 hover:bg-gray-600 text-white opacity-0 group-hover:opacity-100 transition-all duration-200 z-10"
        aria-label="Copy code"
      >
        { }
        {copied ? (
          <Check className="w-4 h-4 text-green-400" />
        ) : (
          <Copy className="w-4 h-4" />
        )}
      </button>
      { }
      <pre className={className}>
        {children}
      </pre>
    </div>
  );
};
CodeBlock.propTypes = {
  children: PropTypes.node.isRequired,
  className: PropTypes.string,
};
export default CodeBlock;
