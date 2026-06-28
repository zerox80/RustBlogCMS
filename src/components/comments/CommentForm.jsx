import React from 'react'
import PropTypes from 'prop-types'
import { Send, Smile } from 'lucide-react'
import EmojiPicker from 'emoji-picker-react'

const CommentForm = ({
    onSubmit,
    newComment,
    setNewComment,
    guestName,
    setGuestName,
    isLoading,
    isAuthenticated,
    showEmojiPicker,
    setShowEmojiPicker,
    onEmojiClick,
    emojiPickerRef
}) => {
    return (
        <form onSubmit={onSubmit} className="mb-8 relative">
            {!isAuthenticated && (
                <div className="mb-4">
                    <label htmlFor="guestName" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        Name (erforderlich)
                    </label>
                    <input
                        type="text"
                        id="guestName"
                        value={guestName}
                        onChange={(e) => setGuestName(e.target.value)}
                        className="w-full md:w-1/2 px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
                        placeholder="Dein Name"
                        maxLength={50}
                        required
                    />
                </div>
            )}

            <div className="relative">
                <textarea
                    value={newComment}
                    onChange={(e) => setNewComment(e.target.value)}
                    placeholder="Schreibe einen Kommentar..."
                    className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 rounded-xl bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-primary-500 resize-none"
                    rows={4}
                    maxLength={1000}
                    required
                />
                <button
                    type="button"
                    onClick={() => setShowEmojiPicker(!showEmojiPicker)}
                    className="absolute bottom-3 right-3 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                >
                    <Smile className="w-6 h-6" />
                </button>
            </div>

            {showEmojiPicker && (
                <div className="absolute z-10 mt-2 right-0" ref={emojiPickerRef}>
                    <EmojiPicker onEmojiClick={onEmojiClick} theme="auto" />
                </div>
            )}

            <div className="mt-2 flex justify-between items-center">
                <span className="text-sm text-gray-500 dark:text-gray-400">
                    {newComment.length}/1000
                </span>
                <button
                    type="submit"
                    disabled={isLoading || !newComment.trim() || (!isAuthenticated && !guestName.trim())}
                    className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                    <Send className="w-4 h-4" />
                    Absenden
                </button>
            </div>
        </form>
    )
}

CommentForm.propTypes = {
    onSubmit: PropTypes.func.isRequired,
    newComment: PropTypes.string.isRequired,
    setNewComment: PropTypes.func.isRequired,
    guestName: PropTypes.string.isRequired,
    setGuestName: PropTypes.func.isRequired,
    isLoading: PropTypes.bool.isRequired,
    isAuthenticated: PropTypes.bool.isRequired,
    showEmojiPicker: PropTypes.bool.isRequired,
    setShowEmojiPicker: PropTypes.func.isRequired,
    onEmojiClick: PropTypes.func.isRequired,
    emojiPickerRef: PropTypes.shape({
        current: PropTypes.any,
    }).isRequired,
}

export default CommentForm
