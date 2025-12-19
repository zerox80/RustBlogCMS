import React, { useState, useEffect } from 'react'
import { useEdit } from '../../context/EditContext'
import { useContent } from '../../context/ContentContext'

/**
 * A highly flexible, inline editable text component integrated with the CMS.
 * 
 * This component automatically switches between a display state (plain text) 
 * and an edit state (input/textarea) based on the global `isEditing` context.
 * It handles its own internal state during editing and commits changes to the 
 * global content state on blur.
 * 
 * @param {Object} props
 * @param {string} props.section - The CMS content section name (e.g., 'hero', 'footer').
 * @param {string} props.field - The specific field path within the section (e.g., 'title', 'features.0.title').
 * @param {string} props.value - The initial fallback value if no CMS data exists.
 * @param {boolean} [props.multiline=false] - Whether to use a textarea instead of a text input.
 * @param {string} [props.className=''] - Additional CSS classes for the wrapper component.
 * @param {React.ElementType} [props.as='span'] - The HTML element or component to render as (e.g., 'h1', 'div').
 */
const EditableText = ({
    section,
    field,
    value,
    multiline = false,
    className = '',
    as: Component = 'span',
    ...props
}) => {
    const { isEditing } = useEdit()
    const { updateSection, getSection } = useContent()
    const [currentValue, setCurrentValue] = useState(value)
    const [internalValue, setInternalValue] = useState(value)

    useEffect(() => {
        setCurrentValue(value)
        setInternalValue(value)
    }, [value])

    /**
     * Commits the edited value back to the CMS backend.
     * 
     * Uses a deep-cloning strategy to preserve data integrity when updating 
     * nested fields (paths like "array.0.field").
     */
    const handleBlur = async () => {
        // Skip update if value hasn't changed
        if (internalValue === currentValue) return

        try {
            // Step 1: Fetch current state of the entire section
            const sectionData = getSection(section)

            // Step 2: Deep clone to avoid direct mutation of the global state object
            const updatedSection = JSON.parse(JSON.stringify(sectionData))

            // Step 3: Traverse the object tree using the dot-notated field path
            const keys = field.split('.')
            let target = updatedSection
            for (let i = 0; i < keys.length - 1; i++) {
                // Ensure intermediate objects exist
                if (!target[keys[i]]) target[keys[i]] = {}
                target = target[keys[i]]
            }

            // Step 4: Set the new value at the leaf node
            target[keys[keys.length - 1]] = internalValue

            // Step 5: Persist via the content context (API call + State Sync)
            await updateSection(section, updatedSection)
        } catch (err) {
            console.error('CMS: Failed to save changes', err)
            // Revert UI to last known good value on failure
            setInternalValue(currentValue)
        }
    }

    return (
        <Component className={`relative ${className}`} {...props}>
            {isEditing ? (
                multiline ? (
                    <textarea
                        value={internalValue}
                        onChange={(e) => setInternalValue(e.target.value)}
                        onBlur={handleBlur}
                        className="w-full bg-slate-800 text-white border border-neon-cyan/50 rounded p-1 outline-none focus:ring-2 focus:ring-neon-cyan/50 min-h-[1.5em] placeholder-slate-400"
                        style={{ WebkitTextFillColor: 'white' }}
                    />
                ) : (
                    <input
                        type="text"
                        value={internalValue}
                        onChange={(e) => setInternalValue(e.target.value)}
                        onBlur={handleBlur}
                        className="w-full bg-slate-800 text-white border border-neon-cyan/50 rounded px-1 outline-none focus:ring-2 focus:ring-neon-cyan/50 min-w-[5ch] placeholder-slate-400"
                        style={{ width: `${Math.max(internalValue.length, 5)}ch`, WebkitTextFillColor: 'white' }}
                    />
                )
            ) : (
                currentValue
            )}
            {isEditing && (
                <span className="absolute -top-1 -right-1 w-2 h-2 bg-neon-cyan rounded-full pointer-events-none" />
            )}
        </Component>
    )
}

export default EditableText
