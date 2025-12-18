import React from 'react'

const LandingStats = ({ stats }) => {
    // Use stats from props
    const displayStats = Array.isArray(stats) ? stats : []

    return (
        <section className="py-20 relative z-10">
            <div className="container px-6 mx-auto">
                <div className="glass-card p-12 rounded-[2.5rem]">
                    <div className="grid grid-cols-2 lg:grid-cols-4 gap-8 text-center divide-x divide-white/10">
                        {displayStats.map((stat, idx) => (
                            <div key={idx} className="flex flex-col items-center p-4">
                                <div className="text-4xl md:text-5xl font-extrabold text-white mb-2 tracking-tight">
                                    {stat.value}
                                </div>
                                <div className="text-slate-400 font-medium uppercase tracking-wider text-sm">
                                    {stat.label}
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
