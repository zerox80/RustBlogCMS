import html2pdf from 'html2pdf.js'

// Create a shared canvas for color conversion
const canvas = document.createElement('canvas')
canvas.width = 1
canvas.height = 1
const ctx = canvas.getContext('2d', { willReadFrequently: true })

/**
 * Forces any color string to a Hex/RGB format using Canvas.
 * This is the most robust way to convert oklch/oklab/etc to something html2canvas understands.
 */
const forceToHex = (colorString) => {
  if (!colorString || colorString === 'none' || colorString === 'transparent') return colorString

  // If it's already safe, return it (optimization)
  if (
    !colorString.includes('oklch') &&
    !colorString.includes('oklab') &&
    !colorString.startsWith('color(')
  ) {
    return colorString
  }

  try {
    ctx.fillStyle = colorString
    // If the browser understands the color, it will set it.
    // If we read it back, we usually get hex or rgb.
    const value = ctx.fillStyle

    // If it returns the same oklch string, the browser didn't convert it (unlikely in modern browsers)
    // or it's just returning the input. In that case, fallback to black or transparent.
    if (value.includes('oklch') || value.includes('oklab')) {
      console.warn('Canvas failed to convert color:', colorString)
      return '#000000' // Fallback
    }
    return value
  } catch (e) {
    console.error('Error converting color:', colorString, e)
    return '#000000'
  }
}

/**
 * Copies computed styles from a source element to a target element.
 * This "bakes" the styles into the element so it looks correct even without stylesheets.
 */
const copyComputedStyles = (source, target) => {
  const computed = window.getComputedStyle(source)

  // We can't just copy cssText because it's often empty for computed styles.
  // We need to copy specific properties. This is a comprehensive list of visual properties.
  const properties = [
    // Layout
    // Layout
    'display',
    'position',
    'margin',
    'padding',
    'top',
    'left',
    'right',
    'bottom',
    'float',
    'clear',
    'z-index',
    'box-sizing',
    'overflow',

    // Flex/Grid (important for layout)
    'flex',
    'flex-direction',
    'flex-wrap',
    'justify-content',
    'align-items',
    'align-content',
    'gap',
    'grid-template-columns',
    'grid-template-rows',
    'grid-gap',

    // Typography
    'font-family',
    'font-size',
    'font-weight',
    'font-style',
    'line-height',
    'text-align',
    'text-transform',
    'text-decoration',
    'letter-spacing',
    'white-space',
    'color',
    'word-break',
    'word-wrap',
    'overflow-wrap',
    'hyphens',

    // Visuals
    'background-color',
    'background-image',
    'background-position',
    'background-size',
    'background-repeat',
    'border',
    'border-top',
    'border-right',
    'border-bottom',
    'border-left',
    'border-radius',
    'border-collapse',
    'border-spacing',
    'box-shadow',
    'opacity',
    'visibility',
  ]

  properties.forEach((prop) => {
    // We use getPropertyValue to get the resolved value
    let value = computed.getPropertyValue(prop)

    if (value && value !== 'none' && value !== 'auto' && value !== 'normal') {
      // FORCE CONVERT COLORS
      if (prop.includes('color') || prop.includes('border') || prop.includes('background')) {
        if (value.includes('oklch') || value.includes('oklab')) {
          value = forceToHex(value)
        }
      }
      target.style.setProperty(prop, value)
    }
  })

  // Explicitly handle background color if it's transparent
  if (
    computed.backgroundColor === 'rgba(0, 0, 0, 0)' ||
    computed.backgroundColor === 'transparent'
  ) {
    // Don't force white, just leave transparent
  } else {
    let bg = computed.backgroundColor
    if (bg.includes('oklch') || bg.includes('oklab')) bg = forceToHex(bg)
    target.style.backgroundColor = bg
  }

  // Explicitly handle color
  let color = computed.color
  if (color.includes('oklch') || color.includes('oklab')) color = forceToHex(color)
  target.style.color = color
}

/**
 * Recursively copies styles from a source tree to a target tree.
 */
const cloneWithStyles = (source, target) => {
  copyComputedStyles(source, target)

  // Iterate children
  for (let i = 0; i < source.children.length; i++) {
    const sourceChild = source.children[i]
    const targetChild = target.children[i]
    if (sourceChild && targetChild) {
      cloneWithStyles(sourceChild, targetChild)
    }
  }
}

/**
 * Generates a PDF from a React element (DOM node).
 * @param {HTMLElement} element - The DOM element to render.
 * @param {string} filename - The output filename.
 */
export const generatePdf = async (element, filename) => {
  if (!element) throw new Error('Element not found')

  // 1. Create a hidden iframe to isolate the PDF generation
  const iframe = document.createElement('iframe')
  iframe.style.position = 'absolute'
  iframe.style.left = '-9999px'
  iframe.style.top = '0'
  iframe.style.width = '800px' // A4 width
  iframe.style.height = '100%'
  iframe.style.border = 'none'
  document.body.appendChild(iframe)

  try {
    const doc = iframe.contentDocument || iframe.contentWindow.document
    doc.open()
    doc.write('<html><head></head><body></body></html>')
    doc.close()

    // 2. Clone the element
    const clone = element.cloneNode(true)

    // 3. Copy computed styles from the original element to the clone
    // This includes the FORCE COLOR CONVERSION logic
    cloneWithStyles(element, clone)

    // 4. Clean up the clone for print (overrides)
    const cleanElement = (el) => {
      // Remove shadows and borders that look bad in print
      if (el.style.boxShadow) el.style.boxShadow = 'none'

      // Force white background for the main container
      if (el.tagName === 'ARTICLE') {
        el.style.backgroundColor = '#ffffff'
        el.style.color = '#000000'
        el.style.border = 'none'
        el.style.width = '100%'
        el.style.maxWidth = '100%'
        el.style.boxSizing = 'border-box'
        el.style.margin = '0'
      }

      // Fix image sizing
      if (el.tagName === 'IMG') {
        el.style.maxWidth = '100%'
        el.style.height = 'auto'
        el.style.pageBreakInside = 'avoid'
      }

      // Ensure code blocks wrap
      if (el.tagName === 'PRE' || el.tagName === 'CODE') {
        el.style.whiteSpace = 'pre-wrap'
        el.style.wordBreak = 'break-word'
        el.style.overflowWrap = 'break-word'
      } else {
        // For all other elements, ensure they don't overflow
        el.style.overflowWrap = 'break-word'
        el.style.wordBreak = 'break-word'
      }

      // Recursively clean
      for (let i = 0; i < el.children.length; i++) {
        cleanElement(el.children[i])
      }
    }
    cleanElement(clone)

    // 5. Append clone to iframe
    doc.body.appendChild(clone)

    // Add some basic reset styles to the iframe body
    doc.body.style.margin = '0'
    doc.body.style.padding = '20px'
    doc.body.style.backgroundColor = '#ffffff'
    doc.body.style.fontFamily = 'sans-serif'
    doc.body.style.textRendering = 'optimizeLegibility'

    // 6. Configure html2pdf
    const opt = {
      margin: [10, 10, 10, 10],
      filename: filename,
      image: { type: 'jpeg', quality: 0.98 },
      html2canvas: {
        scale: 2,
        useCORS: true,
        logging: false,
        backgroundColor: '#ffffff',
        windowWidth: 800,
      },
      jsPDF: { unit: 'mm', format: 'a4', orientation: 'portrait' },
      pagebreak: { mode: ['avoid-all', 'css', 'legacy'] },
    }

    // 7. Generate PDF from the element INSIDE the iframe
    await html2pdf().set(opt).from(clone).save()
  } finally {
    // 8. Cleanup
    document.body.removeChild(iframe)
  }
}
