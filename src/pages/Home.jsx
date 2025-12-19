import { useEffect } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import Hero from '../components/Hero'
import TutorialSection from '../components/TutorialSection'
import { scrollToSection } from '../utils/scrollToSection'

/**
 * The application's homepage component.
 * 
 * Acts as the primary landing point for authenticated or returning users.
 * 
 * Features:
 * - Smart Scroll Restoration: Handles `location.state.scrollTo` to jump to specific sections upon navigation.
 * - Composition: Combines the `Hero` (standard) and `TutorialSection` components.
 */
const Home = () => {
  const location = useLocation()
  const navigate = useNavigate()
  useEffect(() => {
    const targetSection = location.state?.scrollTo
    if (!targetSection) {
      return
    }
    requestAnimationFrame(() => {
      scrollToSection(targetSection)
    })
    navigate(location.pathname, { replace: true, state: {} })
  }, [location, navigate])
  return (
    <>
      <Hero />
      <TutorialSection />
    </>
  )
}
export default Home
