import React, { useState, useEffect } from 'react'
import { useEdit } from '../../context/EditContext'
import { useContent } from '../../context/ContentContext'
import { Image } from 'lucide-react'

const EditableImage = ({
    section,
    field, // e.g. 'heroImage' or 'features[0].icon' (though icon is not image usually)
    src,
    alt,
    className = '',
    containerClassName = '',
    ...props
}) => {
    const { isEditing } = useEdit()
    const { updateSection, getSection } = useContent()
    const [currentSrc, setCurrentSrc] = useState(src)

    useEffect(() => {
        setCurrentSrc(src)
    }, [src])

    const handleEdit = async () => {
        // Simple prompt for now. Ideally this would be a media picker.
        const newUrl = window.prompt("Enter new image URL:", currentSrc)
        if (newUrl && newUrl !== currentSrc) {
            setCurrentSrc(newUrl)

            try {
                const sectionData = getSection(section)

                // Deep clone to avoid mutating state directly
                const updatedSection = JSON.parse(JSON.stringify(sectionData))

                // Helper to set nested value
                // We need to handle nested fields like 'brand.icon' or simple 'heroImage'
                const keys = field.split('.')
                let target = updatedSection
                for (let i = 0; i < keys.length - 1; i++) {
                    if (!target[keys[i]]) target[keys[i]] = {}
                    target = target[keys[i]]
                }
                target[keys[keys.length - 1]] = newUrl

                await updateSection(section, updatedSection)

            } catch (err) {
                console.error('Failed to update image:', err)
                alert('Failed to save image update')
                setCurrentSrc(src) // Revert
            }
        }
    }

    return (
        <div className={`relative group ${containerClassName}`}>
            <img
                src={currentSrc}
                alt={alt}
                className={className}
                {...props}
            />
            {isEditing && (
                <button
                    onClick={(e) => {
                        e.preventDefault()
                        e.stopPropagation()
                        handleEdit()
                    }}
                    className="absolute inset-0 bg-black/50 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer border-2 border-neon-cyan"
                >
                    <div className="bg-slate-900 p-2 rounded-full text-neon-cyan">
                        <Image className="w-6 h-6" />
                    </div>
                </button>
            )}
        </div>
    )
}

export default EditableImage
