import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import TutorialCard from '../TutorialCard'
import { Terminal } from 'lucide-react'

// Mock the scrollToSection utility
vi.mock('../../utils/scrollToSection', () => ({
    scrollToSection: vi.fn(),
}))

import { scrollToSection } from '../../utils/scrollToSection'

describe('TutorialCard', () => {
    const defaultProps = {
        icon: Terminal,
        title: 'Linux Basics',
        description: 'Learn the command line.',
        topics: ['Shell', 'Files'],
        color: 'from-blue-500 to-blue-600',
        onSelect: vi.fn(),
        buttonLabel: 'Start Now',
    }

    it('renders title and description', () => {
        render(<TutorialCard {...defaultProps} />)
        expect(screen.getByText('Linux Basics')).toBeInTheDocument()
        expect(screen.getByText('Learn the command line.')).toBeInTheDocument()
    })

    it('renders topics', () => {
        render(<TutorialCard {...defaultProps} />)
        expect(screen.getByText('Shell')).toBeInTheDocument()
        expect(screen.getByText('Files')).toBeInTheDocument()
    })

    it('calls onSelect when button is clicked', () => {
        render(<TutorialCard {...defaultProps} />)
        const button = screen.getByRole('button')
        fireEvent.click(button)
        expect(defaultProps.onSelect).toHaveBeenCalled()
    })

    it('calls scrollToSection if onSelect is not provided', () => {
        const propsWithoutSelect = { ...defaultProps, onSelect: undefined }
        render(<TutorialCard {...propsWithoutSelect} />)
        const button = screen.getByRole('button')
        fireEvent.click(button)
        expect(scrollToSection).toHaveBeenCalledWith('tutorials')
    })

    it('renders custom button label', () => {
        render(<TutorialCard {...defaultProps} />)
        expect(screen.getByText('Start Now')).toBeInTheDocument()
    })
})
