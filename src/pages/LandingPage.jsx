import React from 'react'
import Header from '../components/layout/Header'
import Footer from '../components/layout/Footer'
import { useContent } from '../context/ContentContext'
import LandingHero from '../components/landing/LandingHero'
import LandingFeatures from '../components/landing/LandingFeatures'
import LandingStats from '../components/landing/LandingStats'
import LandingCTA from '../components/landing/LandingCTA'

/**
 * The high-level root component for the landing page.
 * 
 * Composition:
 * - Composes modular landing sections (Hero, Features, Stats, CTA).
 * - Orchestrates content retrieval from `ContentContext`.
 * - Provides a unified layout container with a dark slate theme.
 */
const LandingPage = () => {
    const { getSection } = useContent()
    const heroContent = getSection('hero') || {}
    const features = Array.isArray(heroContent.features) ? heroContent.features : []
    const statsContent = getSection('stats') || {}
    const statsItems = Array.isArray(statsContent.items) ? statsContent.items : []
    const ctaContent = getSection('cta_section')

    return (
        <div className="min-h-screen bg-slate-950 text-slate-100 font-sans selection:bg-primary-500/30 overflow-x-hidden">
            <Header />
            <LandingHero content={heroContent} />
            <LandingFeatures features={features} section="hero" />
            <LandingStats stats={statsItems} section="stats" />
            <LandingCTA content={ctaContent} section="cta_section" />
            <Footer />
        </div>
    )
}

export default LandingPage
