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

                {/* Floating Glass Cards Visualization */}
                <div className="relative mt-20 h-64 md:h-96 w-full max-w-6xl mx-auto perspective-1000 hidden md:block animate-fade-in [animation-delay:1000ms]">

                    {/* Central Dashboard Mockup */}
                    <div
                        className="absolute inset-x-0 top-0 mx-auto w-3/4 h-full glass-card transform transition-transform duration-200 ease-out hover:scale-105 z-20 overflow-hidden"
                        style={{ transform: `translateY(${mousePosition.y * -1}px) rotateX(5deg)` }}
                    >
                        {/* Browser Bar */}
                        <div className="flex items-center gap-2 px-4 py-3 border-b border-white/5 bg-slate-900/50">
                            <div className="flex gap-1.5">
                                <div className="w-3 h-3 rounded-full bg-red-500/20 border border-red-500/50"></div>
                                <div className="w-3 h-3 rounded-full bg-yellow-500/20 border border-yellow-500/50"></div>
                                <div className="w-3 h-3 rounded-full bg-green-500/20 border border-green-500/50"></div>
                            </div>
                            <div className="mx-auto text-xs text-slate-500 font-mono">dashboard.rs</div>
                        </div>

                        {/* Mock Content */}
                        <div className="p-8 grid grid-cols-3 gap-6">
                            <div className="col-span-2 space-y-4">
                                <div className="h-32 rounded-xl bg-slate-800/50 border border-slate-700/50 animate-pulse-slow relative overflow-hidden">
                                    <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/5 to-transparent skew-x-12 translate-x-[-150%] animate-shimmer" />
                                </div>
                                <div className="flex gap-4">
                                    <div className="h-24 w-1/2 rounded-xl bg-slate-800/30 border border-slate-700/30"></div>
                                    <div className="h-24 w-1/2 rounded-xl bg-slate-800/30 border border-slate-700/30"></div>
                                </div>
                            </div>
                            <div className="space-y-4">
                                <div className="h-20 rounded-xl bg-slate-800/50 border border-slate-700/50"></div>
                                <div className="h-40 rounded-xl bg-neon-violet/5 border border-neon-violet/10 relative p-4">
                                    <div className="w-full h-2 bg-slate-700/50 rounded mb-2"></div>
                                    <div className="w-2/3 h-2 bg-slate-700/50 rounded mb-2"></div>
                                    <div className="w-3/4 h-2 bg-slate-700/50 rounded"></div>
                                    <div className="absolute bottom-4 right-4 w-8 h-8 rounded-full bg-neon-violet/20 animate-bounce"></div>
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* Floating Element Left */}
                    <div
                        className="absolute -left-4 top-20 w-56 p-4 glass-card z-30 animate-float"
                        style={{ transform: `translateY(${mousePosition.y}px)` }}
                    >
                        <div className="flex items-center gap-3 mb-2">
                            <div className="p-2 rounded-lg bg-neon-cyan/10 text-neon-cyan border border-neon-cyan/20">
                                <Zap className="w-5 h-5" />
                            </div>
                            <div>
                                <div className="text-sm font-bold text-white">Blazing Fast</div>
                                <div className="text-xs text-slate-400">100ms Load Time</div>
                            </div>
                        </div>
                    </div>

                    {/* Floating Element Right */}
                    <div
                        className="absolute -right-4 bottom-32 w-56 p-4 glass-card z-30 animate-float-delayed-2s"
                        style={{ transform: `translateY(${mousePosition.y * 1.5}px)` }}
                    >
                        <div className="flex items-center gap-3 mb-2">
                            <div className="p-2 rounded-lg bg-neon-violet/10 text-neon-violet border border-neon-violet/20">
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
