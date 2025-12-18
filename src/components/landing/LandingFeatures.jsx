import React from 'react'
import { Terminal, Cpu, Globe, Lock, Palette, Zap } from 'lucide-react'

const LandingFeatures = ({ features }) => {
    // Use features from props, or empty array if not provided
    const displayFeatures = Array.isArray(features) ? features : []

    return (
        <section id="features" className="py-24 relative overflow-hidden">
            <div className="container px-6 mx-auto relative z-10">
                <div className="text-center max-w-2xl mx-auto mb-16">
                    <h2 className="section-title">
                        Everything you need to <span className="gradient-text-aurora">scale</span>
                    </h2>
                    <p className="text-slate-400 text-lg">
                        Powerful features packaged in a beautiful interface.
                    </p>
                </div>

                {/* Bento Grid Layout */}
                <div className="grid grid-cols-1 md:grid-cols-3 gap-6 auto-rows-[minmax(200px,auto)]">
                    {displayFeatures.map((feature, idx) => (
                        <FeatureCard key={idx} feature={feature} index={idx} />
                    ))}
                </div>
            </div>
        </section>
    )
}

const FeatureCard = ({ feature, index }) => {
    const Icon = feature.icon || Star
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
                <h3 className="text-xl font-bold text-white mb-2">{feature.title}</h3>
                <p className="text-slate-400 leading-relaxed">
                    {feature.description}
                </p>
            </div>

            {/* Decorative gradient blob on hover */}
            <div className={`absolute -right-10 -bottom-10 w-40 h-40 rounded-full blur-3xl opacity-0 group-hover:opacity-20 transition-opacity duration-500 pointer-events-none ${feature.bg?.replace('/10', '/50') || ''}`} />
        </div>
    )
}

import { Shield, Star } from 'lucide-react'

export default LandingFeatures
