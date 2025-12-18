import React, { useState, useEffect } from 'react'
import { Link, useLocation } from 'react-router-dom'
import { Menu, X, Github, Moon, Sun } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useTheme } from '../../context/ThemeContext'

const Header = () => {
    const { t } = useTranslation()
    const { isDarkMode, toggleTheme } = useTheme()
    const [isScrolled, setIsScrolled] = useState(false)
    const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false)
    const location = useLocation()

    useEffect(() => {
        const handleScroll = () => {
            setIsScrolled(window.scrollY > 20)
        }
        window.addEventListener('scroll', handleScroll)
        return () => window.removeEventListener('scroll', handleScroll)
    }, [])

    const navLinks = [
        { name: t('nav.features'), path: '/#features' },
        { name: t('nav.tutorial'), path: '/tutorial/getting-started' },
        { name: t('nav.blog'), path: '/blog' },
        { name: t('nav.about'), path: '/about' },
    ]

    return (
        <header
            className={`
                fixed top-0 left-0 right-0 z-50 transition-all duration-300 flex justify-center px-4
                ${isScrolled ? 'pt-4' : 'pt-6'}
            `}
        >
            <nav
                className={`
                    w-full max-w-5xl rounded-full transition-all duration-300
                    flex items-center justify-between px-6 py-3
                    ${isScrolled
                        ? 'bg-slate-900/80 backdrop-blur-xl border border-white/10 shadow-lg shadow-black/20'
                        : 'bg-transparent border border-transparent'
                    }
                `}
            >
                {/* Logo */}
                <Link to="/" className="flex items-center gap-2 group">
                    <div className="w-8 h-8 rounded-lg bg-gradient-to-tr from-neon-cyan to-neon-violet flex items-center justify-center text-white font-bold text-lg shadow-lg shadow-neon-cyan/20 group-hover:shadow-neon-cyan/40 transition-shadow">
                        R
                    </div>
                    <span className="font-bold text-lg text-white tracking-tight group-hover:text-neon-cyan transition-colors hidden sm:block">
                        RustCMS
                    </span>
                </Link>

                {/* Desktop Navigation */}
                <div className="hidden md:flex items-center gap-8">
                    {navLinks.map((link) => (
                        <Link
                            key={link.name}
                            to={link.path}
                            className="text-sm font-medium text-slate-300 hover:text-white transition-colors relative group"
                        >
                            {link.name}
                            <span className="absolute -bottom-1 left-0 w-0 h-0.5 bg-neon-cyan transition-all duration-300 group-hover:w-full"></span>
                        </Link>
                    ))}
                </div>

                {/* Action Buttons */}
                <div className="hidden md:flex items-center gap-4">
                    <button
                        onClick={toggleTheme}
                        className="p-2 rounded-full hover:bg-white/10 transition-colors text-slate-300 hover:text-white"
                    >
                        {isDarkMode ? <Sun className="w-5 h-5" /> : <Moon className="w-5 h-5" />}
                    </button>
                    <a
                        href="https://github.com/zerox80/LinuxTutorialCMS"
                        target="_blank"
                        rel="noreferrer"
                        className="text-slate-400 hover:text-white transition-colors"
                    >
                        <Github className="w-5 h-5" />
                    </a>
                    <Link
                        to="/login"
                        className="px-5 py-2 rounded-full bg-white/10 hover:bg-white/20 text-white text-sm font-semibold transition-all border border-white/5"
                    >
                        {t('nav.login')}
                    </Link>
                </div>

                {/* Mobile Menu Button */}
                <button
                    className="md:hidden text-white p-2"
                    onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
                >
                    {isMobileMenuOpen ? <X /> : <Menu />}
                </button>
            </nav>

            {/* Mobile Menu Overlay */}
            {isMobileMenuOpen && (
                <div className="absolute top-24 left-4 right-4 bg-slate-900/95 backdrop-blur-xl rounded-2xl border border-white/10 p-6 flex flex-col gap-4 shadow-xl animate-fade-in md:hidden">
                    {navLinks.map((link) => (
                        <Link
                            key={link.name}
                            to={link.path}
                            className="text-lg font-medium text-slate-200 py-2 border-b border-white/5 last:border-0"
                            onClick={() => setIsMobileMenuOpen(false)}
                        >
                            {link.name}
                        </Link>
                    ))}
                    <div className="flex gap-4 mt-4 pt-4 border-t border-white/10">
                        <Link to="/login" className="flex-1 btn-primary text-center py-2">Sign In</Link>
                    </div>
                </div>
            )}
        </header>
    )
}

export default Header
