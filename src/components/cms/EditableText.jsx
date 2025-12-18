import React, { useState, useEffect } from 'react'
import { useEdit } from '../../context/EditContext'
import { useContent } from '../../context/ContentContext'

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

    const handleBlur = async () => {
        if (internalValue === currentValue) return

        try {
            const sectionData = getSection(section)
            // Deep clone to avoid mutating state directly
            const updatedSection = JSON.parse(JSON.stringify(sectionData))

            const keys = field.split('.')
            let target = updatedSection
            for (let i = 0; i < keys.length - 1; i++) {
                if (!target[keys[i]]) target[keys[i]] = {}
                target = target[keys[i]]
            }
            target[keys[keys.length - 1]] = internalValue

            await updateSection(section, updatedSection)
            // On success, currentValue will update via props eventually,
            // but for immediate feedback we can set it here too if we want,
            // though typically we rely on the prop update from parent re-render.
        } catch (err) {
            console.error('Failed to save', err)
            setInternalValue(currentValue) // Revert on error
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
