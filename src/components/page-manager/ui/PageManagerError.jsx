import PropTypes from 'prop-types'
import { AlertCircle } from 'lucide-react'

const PageManagerError = ({ message }) => (
  <div
    className={[
      'flex items-start gap-2 rounded-lg border border-red-200 bg-red-50 p-4',
      'text-sm text-red-700',
    ].join(' ')}
  >
    <AlertCircle className="h-4 w-4" />
    <div>
      <p className="font-semibold">Seiten konnten nicht geladen werden</p>
      <p>{message || 'Unbekannter Fehler'}</p>
    </div>
  </div>
)

PageManagerError.propTypes = {
  message: PropTypes.string,
}

PageManagerError.defaultProps = {
  message: null,
}

export default PageManagerError
