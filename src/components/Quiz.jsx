import { useState } from 'react'
import { Check, X, RotateCcw } from 'lucide-react'
import PropTypes from 'prop-types'
const Quiz = ({ questions }) => {
  const [currentQuestion, setCurrentQuestion] = useState(0)
  const [selectedAnswers, setSelectedAnswers] = useState({})
  const [showResults, setShowResults] = useState(false)
  const [score, setScore] = useState(0)
  const handleAnswer = (questionIndex, answerIndex) => {
    setSelectedAnswers((prev) => ({
      ...prev,
      [questionIndex]: answerIndex,
    }))
  }
  const handleSubmit = () => {
    let correctCount = 0
    questions.forEach((q, idx) => {
      if (selectedAnswers[idx] === q.correctAnswer) {
        correctCount++
      }
    })
    setScore(correctCount)
    setShowResults(true)
  }
  const handleReset = () => {
    setCurrentQuestion(0)
    setSelectedAnswers({})
    setShowResults(false)
    setScore(0)
  }
  const progress = ((currentQuestion + 1) / questions.length) * 100
  const question = questions[currentQuestion]
  if (showResults) {
    const percentage = Math.round((score / questions.length) * 100)
    const passed = percentage >= 70
    return (
      <div className="bg-white dark:bg-gray-800 rounded-2xl p-8 shadow-lg">
        <div className="text-center">
          <div
            className={`inline-flex items-center justify-center w-20 h-20 rounded-full mb-4 ${
              passed ? 'bg-green-100 dark:bg-green-900/30' : 'bg-red-100 dark:bg-red-900/30'
            }`}
          >
            {passed ? (
              <Check className="w-10 h-10 text-green-600 dark:text-green-400" />
            ) : (
              <X className="w-10 h-10 text-red-600 dark:text-red-400" />
            )}
          </div>
          <h3 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">
            {passed ? 'Bestanden!' : 'Nicht bestanden'}
          </h3>
          <p className="text-4xl font-bold text-primary-600 dark:text-primary-400 mb-4">
            {percentage}%
          </p>
          <p className="text-gray-600 dark:text-gray-400 mb-6">
            Du hast {score} von {questions.length} Fragen richtig beantwortet.
          </p>
          <button
            onClick={handleReset}
            className={`flex items-center gap-2 mx-auto px-6 py-3 bg-primary-600 text-white
rounded-xl hover:bg-primary-700 transition-colors`}
          >
            <RotateCcw className="w-4 h-4" />
            Nochmal versuchen
          </button>
        </div>
      </div>
    )
  }
  return (
    <div className="bg-white dark:bg-gray-800 rounded-2xl p-8 shadow-lg">
      {}
      <div className="mb-6">
        <div className="flex justify-between items-center mb-2">
          <span className="text-sm font-medium text-gray-600 dark:text-gray-400">
            Frage {currentQuestion + 1} von {questions.length}
          </span>
          <span className="text-sm font-medium text-primary-600 dark:text-primary-400">
            {Math.round(progress)}%
          </span>
        </div>
        <div className="h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
          <div
            className="h-full bg-primary-600 transition-all duration-300"
            style={{ width: `${progress}%` }}
          />
        </div>
      </div>
      {}
      <h3 className="text-xl font-bold text-gray-900 dark:text-gray-100 mb-6">
        {question.question}
      </h3>
      {}
      <div className="space-y-3 mb-6">
        {question.answers.map((answer, idx) => (
          <button
            key={idx}
            onClick={() => handleAnswer(currentQuestion, idx)}
            className={`w-full text-left p-4 rounded-xl border-2 transition-all ${
              selectedAnswers[currentQuestion] === idx
                ? 'border-primary-600 bg-primary-50 dark:bg-primary-900/20'
                : 'border-gray-200 dark:border-gray-700 hover:border-primary-400 dark:hover:border-primary-500'
            }`}
          >
            <span className="text-gray-900 dark:text-gray-100">{answer}</span>
          </button>
        ))}
      </div>
      {}
      <div className="flex justify-between items-center">
        <button
          onClick={() => setCurrentQuestion((prev) => Math.max(0, prev - 1))}
          disabled={currentQuestion === 0}
          className={`px-4 py-2 text-gray-600 dark:text-gray-400 hover:text-gray-900
dark:hover:text-gray-100 disabled:opacity-50 disabled:cursor-not-allowed`}
        >
          Zurück
        </button>
        {currentQuestion === questions.length - 1 ? (
          <button
            onClick={handleSubmit}
            disabled={Object.keys(selectedAnswers).length !== questions.length}
            className={`px-6 py-2 bg-primary-600 text-white rounded-xl hover:bg-primary-700
disabled:opacity-50 disabled:cursor-not-allowed transition-colors`}
          >
            Auswerten
          </button>
        ) : (
          <button
            onClick={() => setCurrentQuestion((prev) => Math.min(questions.length - 1, prev + 1))}
            disabled={selectedAnswers[currentQuestion] === undefined}
            className={`px-6 py-2 bg-primary-600 text-white rounded-xl hover:bg-primary-700
disabled:opacity-50 disabled:cursor-not-allowed transition-colors`}
          >
            Weiter
          </button>
        )}
      </div>
    </div>
  )
}
Quiz.propTypes = {
  questions: PropTypes.arrayOf(
    PropTypes.shape({
      question: PropTypes.string.isRequired,
      answers: PropTypes.arrayOf(PropTypes.string).isRequired,
      correctAnswer: PropTypes.number.isRequired,
    }),
  ).isRequired,
}
export default Quiz
