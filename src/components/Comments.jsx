import { useState, useEffect, useMemo, useCallback, useRef } from 'react';
import { MessageSquare, ChevronDown } from 'lucide-react';
import { useAuth } from '../context/AuthContext';
import { api } from '../api/client';
import PropTypes from 'prop-types';
import CommentItem from './comments/CommentItem';
import CommentForm from './comments/CommentForm';

const VALID_TUTORIAL_ID = /^[a-zA-Z0-9_.-]+$/;
const COMMENTS_PER_PAGE = 20;

const Comments = ({ tutorialId, postId }) => {
  const [comments, setComments] = useState([]);
  const [newComment, setNewComment] = useState('');
  const [guestName, setGuestName] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [loadingComments, setLoadingComments] = useState(false);
  const [loadError, setLoadError] = useState(null);
  const [offset, setOffset] = useState(0);
  const [hasMore, setHasMore] = useState(true);
  const [sortOrder, setSortOrder] = useState('newest'); // 'newest' or 'top'
  const [showEmojiPicker, setShowEmojiPicker] = useState(false);
  const emojiPickerRef = useRef(null);

  const { isAuthenticated, user } = useAuth();
  const isAdmin = Boolean(user && user.role === 'admin');

  const contextId = useMemo(() => {
    const id = tutorialId || postId;
    if (typeof id !== 'string') return null;
    const trimmed = id.trim();
    if (!trimmed || trimmed.length > 100 || !VALID_TUTORIAL_ID.test(trimmed)) return null;
    return trimmed;
  }, [tutorialId, postId]);

  const isPost = Boolean(postId);

  // Close emoji picker when clicking outside
  useEffect(() => {
    const handleClickOutside = (event) => {
      if (emojiPickerRef.current && !emojiPickerRef.current.contains(event.target)) {
        setShowEmojiPicker(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, []);

  const loadComments = useCallback(async (shouldReset = false) => {
    if (!contextId) {
      setComments([]);
      setLoadError(new Error('Kommentare für diese Ressource sind deaktiviert.'));
      return;
    }

    setLoadingComments(true);
    setLoadError(null);

    try {
      const currentOffset = shouldReset ? 0 : offset;
      let data;

      const params = {
        limit: COMMENTS_PER_PAGE,
        offset: currentOffset,
        sort: sortOrder
      };

      if (isPost) {
        data = await api.listPostComments(contextId, params);
      } else {
        data = await api.listTutorialComments(contextId, params);
      }

      const newComments = Array.isArray(data) ? data : [];

      setComments(prev => {
        if (shouldReset) return newComments;
        const existingIds = new Set(prev.map(c => c.id));
        const uniqueNewComments = newComments.filter(c => !existingIds.has(c.id));
        return [...prev, ...uniqueNewComments];
      });
      setOffset(prev => shouldReset ? newComments.length : prev + newComments.length);
      setHasMore(newComments.length === COMMENTS_PER_PAGE);

    } catch (error) {
      console.error('Failed to load comments:', error);
      if (shouldReset) setComments([]);
      setLoadError(error);
    } finally {
      setLoadingComments(false);
    }
  }, [contextId, isPost, offset, sortOrder]);

  // Initial load and when sort changes
  useEffect(() => {
    loadComments(true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [contextId, sortOrder]);

  const handleLoadMore = () => {
    loadComments(false);
  };

  const onEmojiClick = (emojiObject) => {
    setNewComment(prev => prev + emojiObject.emoji);
    setShowEmojiPicker(false);
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    if (!canComment || !newComment.trim()) return;

    if (!isAuthenticated && !guestName.trim()) return;

    setIsLoading(true);
    try {
      if (isPost) {
        await api.createPostComment(contextId, newComment, isAuthenticated ? null : guestName);
      } else {
        await api.createComment(contextId, newComment);
      }

      setNewComment('');
      setGuestName('');
      // Reset and reload to show the new comment at the top
      await loadComments(true);
    } catch (error) {
      console.error('Failed to post comment:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleDelete = async (commentId) => {
    if (!canManageComments) return;
    if (typeof window !== 'undefined' && !window.confirm('Kommentar wirklich löschen?')) return;

    try {
      await api.deleteComment(commentId);
      // Remove from local state to avoid full reload
      setComments(prev => prev.filter(c => c.id !== commentId));
    } catch (error) {
      console.error('Failed to delete comment:', error);
    }
  };

  const handleVote = async (commentId) => {
    try {
      const updatedComment = await api.voteComment(commentId);
      setComments(prev => prev.map(c => c.id === commentId ? updatedComment : c));
    } catch (error) {
      console.error('Failed to vote:', error);
    }
  };

  // Guests can comment on posts, but only auth users on tutorials
  const canComment = (isAuthenticated || isPost) && contextId;
  // Only admin or (theoretically) author can delete, but here we check admin for deletion button
  const canManageComments = isAdmin;

  return (
    <div className="mt-12 pt-8 border-t border-gray-200 dark:border-gray-700">
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100 flex items-center gap-2">
          <MessageSquare className="w-6 h-6" />
          Kommentare
        </h2>

        <div className="relative">
          <select
            value={sortOrder}
            onChange={(e) => setSortOrder(e.target.value)}
            className="appearance-none bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 py-2 px-4 pr-8 rounded-lg leading-tight focus:outline-none focus:bg-white focus:border-gray-500"
          >
            <option value="newest">Neueste zuerst</option>
            <option value="top">Beste zuerst</option>
          </select>
          <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-gray-700 dark:text-gray-300">
            <ChevronDown className="w-4 h-4" />
          </div>
        </div>
      </div>

      {canComment ? (
        <CommentForm
          onSubmit={handleSubmit}
          newComment={newComment}
          setNewComment={setNewComment}
          guestName={guestName}
          setGuestName={setGuestName}
          isLoading={isLoading}
          isAuthenticated={isAuthenticated}
          showEmojiPicker={showEmojiPicker}
          setShowEmojiPicker={setShowEmojiPicker}
          onEmojiClick={onEmojiClick}
          emojiPickerRef={emojiPickerRef}
        />
      ) : (
        <div className="mb-8 p-4 bg-gray-50 dark:bg-gray-800 rounded-xl text-center">
          <p className="text-gray-600 dark:text-gray-400">
            Bitte melde dich an, um Kommentare zu schreiben.
          </p>
        </div>
      )}

      <div className="space-y-4">
        {comments.length === 0 && !loadingComments ? (
          <p className="text-center text-gray-500 dark:text-gray-400 py-8">
            Noch keine Kommentare. Sei der Erste!
          </p>
        ) : (
          comments.map((comment) => (
            <CommentItem
              key={comment.id}
              comment={comment}
              canManageComments={canManageComments}
              onVote={handleVote}
              onDelete={handleDelete}
            />
          ))
        )}
      </div>

      {loadError && (
        <div className="mt-6 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700 dark:border-red-900/40 dark:bg-red-900/20 dark:text-red-200">
          Kommentare konnten nicht geladen werden.
        </div>
      )}

      {hasMore && (
        <div className="mt-8 text-center">
          <button
            onClick={handleLoadMore}
            disabled={loadingComments}
            className="inline-flex items-center gap-2 px-6 py-2 border border-gray-300 dark:border-gray-600 rounded-lg text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors disabled:opacity-50"
          >
            {loadingComments ? (
              'Laden...'
            ) : (
              <>
                <ChevronDown className="w-4 h-4" />
                Mehr Kommentare laden
              </>
            )}
          </button>
        </div>
      )}
    </div>
  );
};

Comments.propTypes = {
  tutorialId: PropTypes.string,
  postId: PropTypes.string,
};

export default Comments;
