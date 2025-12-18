import React, { useEffect, useState } from 'react'
import { ArrowRight, Terminal, Zap, Shield, Layout } from 'lucide-react'
import { Link } from 'react-router-dom'

const LandingHero = ({ content }) => {
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

    return (
        <div className="relative min-h-[90vh] flex items-center justify-center overflow-hidden pt-20">
            {/* Overlay for depth */}
            <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,rgba(15,23,42,0)_0%,rgba(15,23,42,0.8)_100%)] z-0 pointer-events-none" />

            <div className="container relative z-10 px-6 mx-auto">
                <div className="flex flex-col items-center text-center max-w-5xl mx-auto">
                    {/* Badge */}
                    <div className="animate-fade-in [animation-delay:200ms] mb-6">
                        <span className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-slate-800/50 border border-slate-700 backdrop-blur-md text-sm text-slate-300 font-medium hover:border-slate-500 transition-colors">
                            <span className="w-2 h-2 rounded-full bg-neon-cyan animate-pulse"></span>
                            v2.0 is now live
                        </span>
                    </div>

                    {/* Hero Title */}
                    <h1 className="text-5xl md:text-7xl lg:text-8xl font-display font-extrabold tracking-tight mb-8 animate-slide-up [animation-delay:400ms]">
                        Publish Stories
                        <span className="block mt-2 gradient-text-aurora">
                            That Matter
                        </span>
                    </h1>

                    {/* Subtitle */}
                    <p className="text-lg md:text-xl text-slate-400 max-w-2xl mx-auto mb-10 leading-relaxed animate-slide-up [animation-delay:600ms]">
                        {content?.subtitle || "The modern, high-performance content management system designed for creators, developers, and teams who care about speed and aesthetics."}
                    </p>

                    {/* CTA Buttons */}
                    <div className="flex flex-col sm:flex-row gap-4 animate-slide-up [animation-delay:800ms]">
                        <Link to="/tutorial/getting-started" className="btn-primary group">
                            Start Writing
                            <ArrowRight className="w-4 h-4 transition-transform group-hover:translate-x-1" />
                        </Link>
                        <a href="https://github.com/zerox80/LinuxTutorialCMS" target="_blank" rel="noopener noreferrer" className="btn-secondary">
                            View on GitHub
                        </a>
                    </div>
                </div>

                {/* Hero Image / Dashboard Mockup */}
                <div className="relative mt-20 w-full max-w-6xl mx-auto hidden md:block animate-fade-in [animation-delay:1000ms]">

                    {/* Glowing effect behind the image */}
                    <div className="absolute -inset-1 bg-gradient-to-r from-neon-cyan/20 to-neon-violet/20 rounded-2xl blur-2xl opacity-50"></div>

                    {/* The Image Itself */}
                    <div
                        className="relative rounded-2xl overflow-hidden border border-white/10 shadow-2xl shadow-neon-violet/20 bg-slate-900/50 backdrop-blur-sm transform transition-transform duration-500 hover:scale-[1.01]"
                        style={{ transform: `perspective(1000px) rotateX(${mousePosition.y * 0.5}deg) rotateY(${mousePosition.x * 0.5}deg)` }}
                    >
                        <img
                            src={content?.heroImage || "/dashboard-mockup.png"}
                            alt="Dashboard Preview"
                            className="w-full h-auto object-cover rounded-2xl"
                            onError={(e) => {
                                e.target.onerror = null;
                                e.target.src = "/dashboard-mockup.png"; // Fallback if user upload fails
                            }}
                        />

                        {/* Overlay glare effect */}
                        <div className="absolute inset-0 bg-gradient-to-tr from-white/5 to-transparent pointer-events-none"></div>
                    </div>

                    {/* Floating Badge - "Blazing Fast" */}
                    <div
                        className="absolute -left-12 top-1/4glass-card p-4 rounded-xl border border-white/10 bg-slate-900/60 backdrop-blur-md shadow-xl animate-float z-20 hidden lg:block"
                        style={{ transform: `translateY(${mousePosition.y * 1.5}px)` }}
                    >
                        <div className="flex items-center gap-3">
                            <div className="p-2 rounded-lg bg-neon-cyan/10 text-neon-cyan">
                                <Zap className="w-5 h-5" />
                            </div>
                            <div>
                                <div className="text-sm font-bold text-white">Blazing Fast</div>
                                <div className="text-xs text-slate-400">100ms Load Time</div>
                            </div>
                        </div>
                    </div>

                    {/* Floating Badge - "Rust Secure" */}
                    <div
                        className="absolute -right-12 bottom-1/4 glass-card p-4 rounded-xl border border-white/10 bg-slate-900/60 backdrop-blur-md shadow-xl animate-float-delayed-2s z-20 hidden lg:block"
                        style={{ transform: `translateY(${mousePosition.y * -1.5}px)` }}
                    >
                        <div className="flex items-center gap-3">
                            <div className="p-2 rounded-lg bg-neon-violet/10 text-neon-violet">
                                <Shield className="w-5 h-5" />
                            </div>
                            <div>
                                <div className="text-sm font-bold text-white">Rust Secure</div>
                                <div className="text-xs text-slate-400">Memory Safe Core</div>
                            </div>
                        </div>
                    </div>

                </div>
            </div>
        </div>
    )
}

export default LandingHero
