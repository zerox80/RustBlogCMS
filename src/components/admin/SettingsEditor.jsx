import { useState, useEffect } from 'react'
import { Save, Loader2 } from 'lucide-react'
import { api } from '../../api/client'

const SettingsEditor = () => {
    const [loading, setLoading] = useState(true)
    const [saving, setSaving] = useState(false)
    const [settings, setSettings] = useState({
        pdfEnabled: true
    })
    const [message, setMessage] = useState(null)

    const [availablePages, setAvailablePages] = useState([])

    useEffect(() => {
        fetchSettings()
        fetchPages()
    }, [])

    const fetchPages = async () => {
        try {
            const data = await api.listPages()
            if (data && data.items) {
                setAvailablePages(data.items)
            }
        } catch (error) {
            console.error('Error fetching pages:', error)
        }
    }

    const fetchSettings = async () => {
        try {
            const data = await api.getSiteContentSection('settings')
            if (data) {
                // Merge with defaults in case new settings are added later
                setSettings(prev => ({ ...prev, ...data.content }))
            }
        } catch (error) {
            if (error.status === 404) {
                // If settings don't exist yet, use defaults
                console.log('Settings not found, using defaults')
            } else {
                console.error('Error fetching settings:', error)
            }
        } finally {
            setLoading(false)
        }
    }

    const handleSave = async () => {
        setSaving(true)
        setMessage(null)
        try {
            await api.updateSiteContentSection('settings', settings)
            setMessage({ type: 'success', text: 'Einstellungen gespeichert' })
        } catch (error) {
            console.error('Error saving settings:', error)
            setMessage({ type: 'error', text: 'Fehler beim Speichern' })
        } finally {
            setSaving(false)
        }
    }

    if (loading) {
        return (
            <div className="flex justify-center p-8">
                <Loader2 className="h-8 w-8 animate-spin text-blue-600" />
            </div>
        )
    }

    return (
        <div className="space-y-6">
            <div className="bg-white dark:bg-slate-900 rounded-lg shadow p-6">
                <h2 className="text-lg font-medium text-gray-900 dark:text-white mb-6">
                    Globale Einstellungen
                </h2>

                <div className="space-y-6">
                    {/* PDF Settings */}
                    <div className="flex items-center justify-between">
                        <div>
                            <h3 className="text-base font-medium text-gray-900 dark:text-white">
                                PDF Download
                            </h3>
                            <p className="text-sm text-gray-500 dark:text-gray-400">
                                Erlaubt Benutzern das Herunterladen von Tutorials als PDF.
                            </p>
                        </div>
                        <button
                            onClick={() => setSettings(prev => ({ ...prev, pdfEnabled: !prev.pdfEnabled }))}
                            className={`relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-blue-600 focus:ring-offset-2 ${settings.pdfEnabled ? 'bg-blue-600' : 'bg-gray-200 dark:bg-gray-700'
                                }`}
                            role="switch"
                            aria-checked={settings.pdfEnabled}
                        >
                            <span
                                aria-hidden="true"
                                className={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${settings.pdfEnabled ? 'translate-x-5' : 'translate-x-0'
                                    }`}
                            />
                        </button>
                    </div>

                    {/* Start Page Settings */}
                    <div className="pt-6 border-t border-gray-200 dark:border-gray-700">
                        <div className="flex flex-col gap-4">
                            <div>
                                <h3 className="text-base font-medium text-gray-900 dark:text-white">
                                    Startseite
                                </h3>
                                <p className="text-sm text-gray-500 dark:text-gray-400">
                                    Wähle aus, welche Seite beim Aufruf der Hauptdomain (/) angezeigt werden soll.
                                </p>
                            </div>
                            <div className="w-full max-w-md">
                                <select
                                    value={settings.homePageSlug || ''}
                                    onChange={(e) => setSettings(prev => ({ ...prev, homePageSlug: e.target.value }))}
                                    className="block w-full rounded-md border-gray-300 dark:border-gray-600 dark:bg-slate-800 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm p-2.5"
                                >
                                    <option value="">Standard Landing Page</option>
                                    <option value="blog">Blog (Alle Artikel)</option>
                                    <optgroup label="CMS Seiten">
                                        {availablePages.map(page => (
                                            <option key={page.id} value={page.slug}>
                                                {page.title} ({page.slug})
                                            </option>
                                        ))}
                                    </optgroup>
                                </select>
                            </div>
                        </div>
                    </div>
                </div>

                <div className="mt-8 flex items-center justify-end gap-4">
                    {message && (
                        <span className={`text-sm ${message.type === 'success' ? 'text-green-600' : 'text-red-600'
                            }`}>
                            {message.text}
                        </span>
                    )}
                    <button
                        onClick={handleSave}
                        disabled={saving}
                        className="inline-flex items-center gap-2 px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
                    >
                        {saving ? (
                            <>
                                <Loader2 className="h-4 w-4 animate-spin" />
                                Speichern...
                            </>
                        ) : (
                            <>
                                <Save className="h-4 w-4" />
                                Speichern
                            </>
                        )}
                    </button>
                </div>
            </div>
        </div>
    )
}

export default SettingsEditor
