import React, { useEffect, useState } from 'react'
import { ArrowRight, Terminal, Zap, Shield, Layout } from 'lucide-react'
import EditableText from '../cms/EditableText'
import EditableImage from '../cms/EditableImage'
import { Link, useNavigate, useLocation } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { navigateContentTarget } from '../../utils/contentNavigation'

const LandingHero = ({ content }) => {
    const { t } = useTranslation()
    const navigate = useNavigate()
    const location = useLocation()
    const [mousePosition, setMousePosition] = useState({ x: 0, y: 0 })

    useEffect(() => {
        const handleMouseMove = (e) => {
            setMousePosition({
                x: (e.clientX / window.innerWidth) * 20,
                y: (e.clientY / window.innerHeight) * 20,
            })
        }
        window.addEventListener('mousemove', handleMouseMove)
        return () => window.removeEventListener('mousemove', handleMouseMove)
    }, [])

    // Safe defaults from content or fallbacks
    const titleLine1 = content?.title?.line1 || t('hero.title') || "Publish Stories"
    const titleLine2 = content?.title?.line2 || t('hero.subtitle') || "That Matter"
    const subtitle = content?.subtitle || t('hero.description')
    const primaryCta = content?.primaryCta?.label || t('hero.cta_primary') || "Explore Tutorials"
    const secondaryCta = content?.secondaryCta?.label || t('hero.cta_secondary') || "GitHub"
    const heroImage = content?.heroImage || "/hero-dashboard-v2.png"

    return (
        <div className="relative min-h-[90vh] flex items-center justify-center overflow-hidden pt-20">
            {/* Overlay for depth */}
            <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,rgba(15,23,42,0)_0%,rgba(15,23,42,0.8)_100%)] z-0 pointer-events-none" />

            <div className="container relative z-10 px-6 mx-auto">
                <div className="flex flex-col items-center text-center max-w-5xl mx-auto">

                    {/* Hero Title */}
                    <h1 className="text-5xl md:text-7xl lg:text-8xl font-display font-extrabold tracking-tight mb-8 animate-slide-up [animation-delay:400ms]">
                        <EditableText section="hero" field="title.line1" value={titleLine1} />
                        <EditableText
                            section="hero"
                            field="title.line2"
                            value={titleLine2}
                            className="block mt-2 gradient-text-aurora"
                        />
                    </h1>

                    {/* Subtitle */}
                    <p className="text-lg md:text-xl text-slate-400 max-w-2xl mx-auto mb-10 leading-relaxed animate-slide-up [animation-delay:600ms]">
                        <EditableText section="hero" field="subtitle" value={subtitle} multiline />
                    </p>

                    {/* CTA Buttons */}
                    <div className="flex flex-col sm:flex-row gap-4 animate-slide-up [animation-delay:800ms]">
                        <button
                            onClick={() => navigateContentTarget(content?.primaryCta?.target, { navigate, location })}
                            className="btn-primary group"
                        >
                            <EditableText section="hero" field="primaryCta.label" value={primaryCta} />
                            <ArrowRight className="w-4 h-4 transition-transform group-hover:translate-x-1" />
                        </button>
                        <button
                            onClick={() => navigateContentTarget(content?.secondaryCta?.target, { navigate, location })}
                            className="btn-secondary"
                        >
                            <EditableText section="hero" field="secondaryCta.label" value={secondaryCta} />
                        </button>
                    </div>
                </div>

                {/* Hero Image / Dashboard Mockup */}
                <div className="relative mt-20 w-full max-w-3xl mx-auto hidden md:block animate-fade-in [animation-delay:1000ms]">

                    {/* Glowing effect behind the image */}
                    <div className="absolute -inset-1 bg-gradient-to-r from-neon-cyan/20 to-neon-violet/20 rounded-2xl blur-2xl opacity-50"></div>

                    {/* The Image Itself */}
                    <div
                        className="relative rounded-2xl overflow-hidden border border-white/10 shadow-2xl shadow-neon-violet/20 bg-slate-900/50 backdrop-blur-sm transform transition-transform duration-500 hover:scale-[1.01]"
                        style={{ transform: `perspective(1000px) rotateX(${mousePosition.y * 0.5}deg) rotateY(${mousePosition.x * 0.5}deg)` }}
                    >
                        <EditableImage
                            section="hero"
                            field="heroImage"
                            src={heroImage}
                            alt="Dashboard Preview"
                            className="w-full h-auto max-h-[60vh] object-contain rounded-2xl"
                            containerClassName="w-full h-full"
                        />

                        {/* Overlay glare effect */}
                        <div className="absolute inset-0 bg-gradient-to-tr from-white/5 to-transparent pointer-events-none"></div>
                    </div>

                    {content?.badgeText && (
                        <div
                            className="absolute -left-12 top-1/4glass-card p-4 rounded-xl border border-white/10 bg-slate-900/60 backdrop-blur-md shadow-xl animate-float z-20 hidden lg:block"
                            style={{ transform: `translateY(${mousePosition.y * 1.5}px)` }}
                        >
                            <div className="flex items-center gap-3">
                                <div className="p-2 rounded-lg bg-neon-cyan/10 text-neon-cyan">
                                    <Zap className="w-5 h-5" />
                                </div>
                                <div>
                                    <div className="text-sm font-bold text-white">{content.badgeText}</div>
                                </div>
                            </div>
                        </div>
                    )}

                </div>
            </div>
        </div>
    )
}

export default LandingHero
