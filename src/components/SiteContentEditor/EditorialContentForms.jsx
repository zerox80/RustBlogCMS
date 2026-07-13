import PropTypes from 'prop-types'

const fieldClassName = `mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`

export const AboutContentForm = ({ content, onFieldChange }) => {
  const paragraphs = Array.isArray(content?.paragraphs) ? content.paragraphs : ['', '']

  return (
    <div className="space-y-5 rounded-xl border border-gray-200 bg-white p-5 dark:border-slate-700 dark:bg-slate-900">
      <div>
        <label htmlFor="about-eyebrow" className="text-sm font-medium text-gray-700 dark:text-gray-300">
          Bereichsüberschrift
        </label>
        <input
          id="about-eyebrow"
          type="text"
          className={fieldClassName}
          value={content?.eyebrow || ''}
          onChange={(event) => onFieldChange(['eyebrow'], event.target.value)}
        />
      </div>
      <div>
        <label htmlFor="about-lead" className="text-sm font-medium text-gray-700 dark:text-gray-300">
          Leitgedanke
        </label>
        <textarea
          id="about-lead"
          rows="3"
          className={fieldClassName}
          value={content?.lead || ''}
          onChange={(event) => onFieldChange(['lead'], event.target.value)}
        />
      </div>
      {paragraphs.slice(0, 2).map((paragraph, index) => (
        <div key={`about-paragraph-${index}`}>
          <label
            htmlFor={`about-paragraph-${index}`}
            className="text-sm font-medium text-gray-700 dark:text-gray-300"
          >
            Absatz {index + 1}
          </label>
          <textarea
            id={`about-paragraph-${index}`}
            rows="4"
            className={fieldClassName}
            value={paragraph || ''}
            onChange={(event) => onFieldChange(['paragraphs', index], event.target.value)}
          />
        </div>
      ))}
    </div>
  )
}

AboutContentForm.propTypes = {
  content: PropTypes.object,
  onFieldChange: PropTypes.func.isRequired,
}
