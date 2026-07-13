import PropTypes from 'prop-types'

export const LoginForm = ({ content, onFieldChange }) => {
  const loginContent = content || {}
  return (
    <div
      className={`rounded-2xl border border-gray-200 bg-white p-6 shadow-sm
dark:border-slate-700 dark:bg-slate-900/80`}
    >
      <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">Login Seite</h3>
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <div>
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Titel</label>
          <input
            type="text"
            className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
            value={loginContent.title || ''}
            onChange={(e) => onFieldChange(['title'], e.target.value)}
          />
        </div>
        <div>
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Untertitel</label>
          <input
            type="text"
            className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
            value={loginContent.subtitle || ''}
            onChange={(e) => onFieldChange(['subtitle'], e.target.value)}
          />
        </div>
        <div>
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Icon (Lucide Name)
          </label>
          <input
            type="text"
            className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
            value={loginContent.icon || ''}
            onChange={(e) => onFieldChange(['icon'], e.target.value)}
          />
        </div>
        <div>
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Button Label
          </label>
          <input
            type="text"
            className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
            value={loginContent.buttonLabel || ''}
            onChange={(e) => onFieldChange(['buttonLabel'], e.target.value)}
          />
        </div>
        <div>
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Benutzername Label
          </label>
          <input
            type="text"
            className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
            value={loginContent.usernameLabel || ''}
            onChange={(e) => onFieldChange(['usernameLabel'], e.target.value)}
          />
        </div>
        <div>
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Passwort Label
          </label>
          <input
            type="text"
            className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
            value={loginContent.passwordLabel || ''}
            onChange={(e) => onFieldChange(['passwordLabel'], e.target.value)}
          />
        </div>
        <div className="md:col-span-2">
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Zurück Link Text
          </label>
          <input
            type="text"
            className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
            value={loginContent.backLinkText || ''}
            onChange={(e) => onFieldChange(['backLinkText'], e.target.value)}
          />
        </div>
      </div>
    </div>
  )
}

LoginForm.propTypes = {
  content: PropTypes.object,
  onFieldChange: PropTypes.func.isRequired,
}

export const SiteMetaForm = ({ content, onFieldChange }) => {
  const siteMeta = content || {}
  return (
    <div
      className={`rounded-2xl border border-gray-200 bg-white p-6 shadow-sm
dark:border-slate-700 dark:bg-slate-900/80`}
    >
      <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
        Seitentitel & Beschreibung
      </h3>
      <div className="space-y-4">
        <div>
          <label
            className="block text-sm font-medium text-gray-700 dark:text-gray-300"
            htmlFor="site-meta-title"
          >
            Browser-Titel
          </label>
          <input
            id="site-meta-title"
            type="text"
            className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
text-gray-900 focus:border-primary-500 focus:ring-2 focus:ring-primary-200
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100
dark:placeholder:text-slate-400`}
            value={siteMeta.title || ''}
            onChange={(event) => onFieldChange(['title'], event.target.value)}
            placeholder="z. B. IT Portal"
          />
        </div>
        <div>
          <label
            className="block text-sm font-medium text-gray-700 dark:text-gray-300"
            htmlFor="site-meta-description"
          >
            Meta-Beschreibung
          </label>
          <textarea
            id="site-meta-description"
            className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
text-gray-900 focus:border-primary-500 focus:ring-2 focus:ring-primary-200
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100
dark:placeholder:text-slate-400`}
            rows="3"
            value={siteMeta.description || ''}
            onChange={(event) => onFieldChange(['description'], event.target.value)}
            placeholder="Kurze Beschreibung, die in Suchergebnissen angezeigt wird"
          />
          <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
            Empfehlung: 50–160 Zeichen, enthält wichtige Schlüsselbegriffe.
          </p>
        </div>
      </div>
    </div>
  )
}

SiteMetaForm.propTypes = {
  content: PropTypes.object,
  onFieldChange: PropTypes.func.isRequired,
}
