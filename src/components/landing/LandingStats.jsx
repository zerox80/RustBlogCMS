import React from 'react'
import EditableText from '../cms/EditableText'

/**
 * A horizontal statistics bar with glassmorphism styling.
 * 
 * Uses a responsive grid that adjusts from 2 to 4 columns based on screen size.
 * Features a custom `divide-x` border styling to separate stat items.
 * 
 * @param {Object} props
 * @param {Array} props.stats - Array of stat objects { value, label }.
 * @param {string} [props.section='stats'] - CMS section identifier.
 */
const LandingStats = ({ stats, section = 'stats' }) => {
    // Robustness: Ensure stats is an array to avoid crashes if content is missing
    const displayStats = Array.isArray(stats) ? stats : []

    return (
        <section className="py-20 relative z-10">
            <div className="container px-6 mx-auto">
                <div className="glass-card p-12 rounded-[2.5rem]">
                    <div className="grid grid-cols-2 lg:grid-cols-4 gap-8 text-center divide-x divide-white/10">
                        {displayStats.map((stat, idx) => (
                            <div key={idx} className="flex flex-col items-center p-4">
                                <div className="text-4xl md:text-5xl font-extrabold text-white mb-2 tracking-tight">
                                    <EditableText section={section} field={`items.${idx}.value`} value={stat.value} />
                                </div>
                                <div className="text-slate-400 font-medium uppercase tracking-wider text-sm">
                                    <EditableText section={section} field={`items.${idx}.label`} value={stat.label} />
                                </div>
                            </div>
                        ))}
                    </div>
                </div>
            </div>
        </section>
    )
}

export default LandingStats
