import React from 'react'
import { useNavigate } from 'react-router-dom'
import EditableText from '../cms/EditableText'

/**
 * The final "Call to Action" section for the landing page.
 * 
 * Features a high-contrast design with an animated aurora background
 * to draw the user's attention towards the primary conversion button.
 * 
 * @param {Object} props
 * @param {Object} props.content - Content object containing title and description.
 * @param {string} [props.section='cta_section'] - CMS section identifier.
 */
const LandingCTA = ({ content, section = 'cta_section' }) => {
    const navigate = useNavigate()
    const ctaContent = content || {}

    return (
        <section className="py-32 relative overflow-hidden">
            <div className="absolute inset-0 bg-gradient-to-b from-slate-950 to-primary-950/30" />
            <div className="absolute inset-0 aurora-bg opacity-20" />

            <div className="relative max-w-5xl mx-auto px-4 text-center">
                <h2 className="text-5xl md:text-7xl font-bold mb-8 text-white tracking-tight">
                    <EditableText section={section} field="title" value={ctaContent.title} />
                </h2>
                <p className="text-xl md:text-2xl text-slate-300 mb-12 max-w-3xl mx-auto">
                    <EditableText section={section} field="description" value={ctaContent.description} multiline />
                </p>
                <button
                    onClick={() => navigate('/blog')}
                    className="px-12 py-6 bg-white text-slate-950 rounded-full font-bold text-xl transition-all duration-300 hover:scale-105 hover:shadow-[0_0_60px_-15px_rgba(255,255,255,0.4)]"
                >
                    <EditableText section={section} field="buttonLabel" value="Get Started Now" />
                </button>
            </div>
        </section>
    )
}

export default LandingCTA
