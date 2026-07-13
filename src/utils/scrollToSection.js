export const scrollToSection = (sectionId, behavior = 'smooth') => {
  // Server-side rendering check - prevent execution during SSR
  if (typeof window === 'undefined') {
    return
  }
  // Input validation
  if (!sectionId || typeof sectionId !== 'string') {
    return
  }
  // Special case: home navigation
  if (sectionId === 'home') {
    window.scrollTo({ top: 0, behavior })
    return
  }
  // Normalize section identifier for consistent matching
  const normalizedId = sectionId.trim().toLowerCase()
  // Section mapping for German content aliases
  // Maps common German terms to actual section IDs
  const sectionMap = {
    grundlagen: 'tutorials', // German for "basics"
    befehle: 'tutorials', // German for "commands"
    praxis: 'tutorials', // German for "practice"
    advanced: 'tutorials', // English advanced content
    tutorials: 'tutorials', // Direct reference
  }
  // Resolve target identifier through mapping
  const targetIdentifier = sectionMap[normalizedId] || normalizedId
  // Element selection with fallback strategy
  const targetElement =
    // Priority 1: Data attribute (most flexible)
    document.querySelector(`[data-section="${targetIdentifier}"]`) ||
    // Priority 2: Element ID (standard HTML)
    document.getElementById(targetIdentifier)
  // Perform scrolling if element is found
  targetElement?.scrollIntoView({ behavior, block: 'start' })
}
