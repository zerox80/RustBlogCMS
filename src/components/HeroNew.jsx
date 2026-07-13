import { Terminal, Code, Book, Zap, Sparkles, ArrowRight } from 'lucide-react'
const Hero = () => {
  return (
    <section
      className={`relative min-h-[90vh] flex items-center justify-center overflow-hidden
bg-gradient-to-br from-primary-600 via-primary-700 to-indigo-900`}
    >
      {}
      <div className="absolute inset-0 overflow-hidden">
        <div className="absolute inset-0 opacity-20">
          <div
            className={`absolute top-0 -left-4 w-96 h-96 bg-purple-500 rounded-full
mix-blend-multiply filter blur-3xl animate-float`}
          ></div>
          <div
            className={`absolute top-0 -right-4 w-96 h-96 bg-yellow-500 rounded-full
mix-blend-multiply filter blur-3xl animate-float`}
            style={{ animationDelay: '2s' }}
          ></div>
          <div
            className={`absolute -bottom-8 left-20 w-96 h-96 bg-pink-500 rounded-full
mix-blend-multiply filter blur-3xl animate-float`}
            style={{ animationDelay: '4s' }}
          ></div>
        </div>
        {}
        <div
          className="absolute inset-0 opacity-10"
          style={{
            backgroundImage: 'radial-gradient(circle, rgba(255,255,255,0.45) 1px, transparent 1px)',
            backgroundSize: '60px 60px',
          }}
        ></div>
      </div>
      <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-20 z-10">
        <div className="text-center">
          {}
          <div
            className={`inline-flex items-center gap-2 px-4 py-2 rounded-full bg-white/10
backdrop-blur-xl border border-white/20 text-white mb-8 animate-slide-down`}
          >
            <Sparkles className="w-4 h-4" />
            <span className="text-sm font-medium">Professionelles Linux Training</span>
          </div>
          {}
          <div
            className={`inline-flex items-center justify-center w-24 h-24 mb-8 relative group
animate-scale-in`}
          >
            <div
              className={`absolute inset-0 bg-gradient-to-r from-white/20 to-white/10 rounded-3xl
rotate-6 group-hover:rotate-12 transition-transform duration-500`}
            ></div>
            <div
              className={`absolute inset-0 bg-gradient-to-r from-white/10 to-white/5 rounded-3xl
-rotate-6 group-hover:-rotate-12 transition-transform duration-500`}
            ></div>
            <div
              className={`relative bg-white/20 backdrop-blur-xl rounded-2xl p-5 border border-white/30
shadow-2xl`}
            >
              <Terminal className="w-12 h-12 text-white" />
            </div>
          </div>
          {}
          <h1 className="text-5xl sm:text-6xl lg:text-7xl font-bold mb-6 animate-slide-up">
            <span className="text-white drop-shadow-2xl">Lerne Linux</span>
            <br />
            <span
              className={`bg-gradient-to-r from-yellow-200 via-yellow-100 to-yellow-200 bg-clip-text
text-transparent drop-shadow-lg animate-gradient bg-[length:200%_auto]`}
            >
              von Grund auf
            </span>
          </h1>
          {}
          <p
            className={`text-xl sm:text-2xl text-blue-50 mb-12 max-w-3xl mx-auto leading-relaxed
animate-slide-up`}
            style={{ animationDelay: '0.1s' }}
          >
            Dein umfassendes Tutorial für Linux - von den Basics bis zu Advanced Techniken.
            <span className="block mt-2 text-blue-100">Interaktiv, modern und praxisnah.</span>
          </p>
          {}
          <div
            className={`flex flex-col sm:flex-row gap-4 justify-center items-center mb-16
animate-slide-up`}
            style={{ animationDelay: '0.2s' }}
          >
            <button
              className={`group relative px-8 py-4 bg-white text-primary-700 rounded-xl font-semibold
shadow-2xl hover:shadow-white/20 transition-all duration-300 hover:scale-105
hover:-translate-y-1 overflow-hidden`}
            >
              <span className="relative z-10 flex items-center gap-2">
                Los geht&apos;s
                <ArrowRight className="w-5 h-5 group-hover:translate-x-1 transition-transform" />
              </span>
              <div
                className={`absolute inset-0 bg-gradient-to-r from-yellow-100 to-white opacity-0
group-hover:opacity-100 transition-opacity duration-300`}
              ></div>
            </button>
            <button
              className={`group px-8 py-4 bg-white/10 backdrop-blur-xl text-white rounded-xl
font-semibold border-2 border-white/30 hover:bg-white/20
hover:border-white/50 transition-all duration-300 hover:scale-105
hover:-translate-y-1`}
            >
              <span className="flex items-center gap-2">
                Mehr erfahren
                <Book className="w-5 h-5 group-hover:rotate-12 transition-transform" />
              </span>
            </button>
          </div>
          {}
          <div
            className="grid grid-cols-1 sm:grid-cols-3 gap-6 max-w-5xl mx-auto animate-slide-up"
            style={{ animationDelay: '0.3s' }}
          >
            {[
              {
                icon: Book,
                title: 'Schritt für Schritt',
                desc: 'Strukturiert lernen mit klaren Beispielen',
                color: 'from-blue-500 to-cyan-500',
              },
              {
                icon: Code,
                title: 'Praktische Befehle',
                desc: 'Direkt anwendbare Kommandos',
                color: 'from-purple-500 to-pink-500',
              },
              {
                icon: Zap,
                title: 'Modern & Aktuell',
                desc: 'Neueste Best Practices',
                color: 'from-orange-500 to-red-500',
              },
            ].map((feature, i) => (
              <div
                key={i}
                className={`group relative bg-white/10 backdrop-blur-xl rounded-2xl p-6 border
border-white/20 hover:bg-white/20 hover:border-white/40 transition-all
duration-500 hover:scale-105 hover:-translate-y-2 cursor-pointer`}
              >
                {}
                <div
                  className={[
                    'absolute inset-0 rounded-2xl bg-gradient-to-br opacity-0 blur-xl',
                    'transition-opacity duration-500 group-hover:opacity-10',
                    feature.color,
                  ].join(' ')}
                ></div>
                <div className="relative">
                  <div
                    className={`inline-flex p-3 rounded-xl bg-gradient-to-br ${feature.color} mb-4 shadow-lg`}
                  >
                    <feature.icon className="w-6 h-6 text-white" />
                  </div>
                  <h3 className="font-bold text-lg text-white mb-2">{feature.title}</h3>
                  <p className="text-blue-100 text-sm leading-relaxed">{feature.desc}</p>
                </div>
              </div>
            ))}
          </div>
          {}
          <div className="absolute bottom-8 left-1/2 transform -translate-x-1/2 animate-bounce">
            <div
              className={`w-6 h-10 rounded-full border-2 border-white/30 flex items-start
justify-center p-2`}
            >
              <div className="w-1 h-3 bg-white rounded-full"></div>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
export default Hero
