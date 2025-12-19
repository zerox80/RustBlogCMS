import React from 'react'
import { Terminal, Cpu, Globe, Lock, Palette, Zap } from 'lucide-react'

import EditableText from '../cms/EditableText'

/**
 * Displays a feature grid on the landing page using a modern Bento layout.
 * 
 * This component is highly dynamic; it accepts a `features` array from the CMS
 * and maps them into interactive FeatureCards. It uses `EditableText` extensively 
 * to allow real-time content updates.
 * 
 * @param {Object} props
 * @param {Array} props.features - Array of feature objects (title, description, icon, bg, color).
 * @param {string} [props.section='hero'] - The CMS section where these features are stored.
 */
const LandingFeatures = ({ features, section = 'hero' }) => {
    // Robustness: Ensure features is always an array to prevent mapping errors
    const displayFeatures = Array.isArray(features) ? features : []

    return (
        <section id="features" className="py-24 relative overflow-hidden">
            <div className="container px-6 mx-auto relative z-10">
                <div className="text-center max-w-2xl mx-auto mb-16">
                    <h2 className="section-title">
                        <EditableText
                            section="hero"
                            field="features_title"
                            value="Everything you need to "
                            className="text-white"
                        />
                        <EditableText
                            section="hero"
                            field="features_highlight"
                            value="scale"
                            className="gradient-text-aurora ml-2"
                        />
                    </h2>
                    <p className="text-slate-400 text-lg">
                        <EditableText section="hero" field="features_subtitle" value="Powerful features packaged in a beautiful interface." />
                    </p>
                </div>

                {/* Bento Grid Layout */}
                <div className="grid grid-cols-1 md:grid-cols-3 gap-6 auto-rows-[minmax(200px,auto)]">
                    {displayFeatures.map((feature, idx) => (
                        <FeatureCard key={idx} feature={feature} index={idx} section={section} />
                    ))}
                </div>
            </div>
        </section>
    )
}

/**
 * An individual card within the Bento grid.
 * 
 * Features micro-animations, glassmorphism styling, and dynamic color/icon injection.
 * 
 * @param {Object} props
 * @param {Object} props.feature - The feature data object.
 * @param {number} props.index - The index in the array (used for staggered animations).
 * @param {string} props.section - Parent section name for CMS updates.
 */
const FeatureCard = ({ feature, index, section }) => {
    // Default to 'Star' icon if specific icon name is missing or invalid
    const Icon = feature.icon || Star

    // Layout Logic: Large features span two columns in the grid
    const isLarge = feature.size === 'large'

    return (
        <div
            className={`
                group glass-card p-8 flex flex-col justify-between 
                ${isLarge ? 'md:col-span-2' : 'md:col-span-1'}
                hover:border-white/20 transition-all duration-300
            `}
            style={{ animationDelay: `${index * 100}ms` }}
        >
            <div className="mb-6">
                <div className={`w-12 h-12 rounded-xl flex items-center justify-center mb-4 ${feature.bg} ${feature.color} group-hover:scale-110 transition-transform duration-300`}>
                    <Icon className="w-6 h-6" />
                </div>
                <h3 className="text-xl font-bold text-white mb-2">
                    <EditableText section={section} field={`features.${index}.title`} value={feature.title} />
                </h3>
                <p className="text-slate-400 leading-relaxed">
                    <EditableText section={section} field={`features.${index}.description`} value={feature.description} multiline />
                </p>
            </div>

            {/* Decorative gradient blob on hover */}
            <div className={`absolute -right-10 -bottom-10 w-40 h-40 rounded-full blur-3xl opacity-0 group-hover:opacity-20 transition-opacity duration-500 pointer-events-none ${feature.bg?.replace('/10', '/50') || ''}`} />
        </div>
    )
}

import { Shield, Star } from 'lucide-react'

export default LandingFeatures
