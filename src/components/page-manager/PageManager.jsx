import PageList from './PageList'
import PageStats from './PageStats'
import PostsPanel from './PostsPanel'
import usePageManagerState from './hooks/usePageManagerState'
import PageManagerHeader from './ui/PageManagerHeader'
import PageManagerError from './ui/PageManagerError'
import PageManagerModals from './ui/PageManagerModals'
import PageManagerEmptyState from './ui/PageManagerEmptyState'

const PageManager = () => {
  const {
    navigation,
    publishedSlugList,
    pages,
    loading,
    error,
    selectedPage,
    selectedPageId,
    setSelectedPageId,
    posts,
    postsLoading,
    postsError,
    pageForm,
    postForm,
    pagesActions,
    postsActions,
  } = usePageManagerState()

  return (
    <div className="space-y-8">
      <PageManagerHeader
        onRefresh={pagesActions.refresh}
        refreshing={loading}
        onExportMarkdown={pagesActions.exportMarkdown}
        markdownExporting={pagesActions.markdownExporting}
        onCreate={pageForm.openCreate}
      />
      <PageStats
        navigation={navigation}
        publishedSlugs={publishedSlugList}
        pages={pages}
        selectedPage={selectedPage}
      />
      {error && <PageManagerError message={error?.message} />}
      <div className="grid gap-6 lg:grid-cols-2">
        <PageList
          pages={pages}
          loading={loading}
          selectedPageId={selectedPageId}
          onSelect={setSelectedPageId}
          onEdit={pageForm.openEdit}
          onDelete={pagesActions.delete}
        />
        <div>
          {selectedPage ? (
            <PostsPanel
              page={selectedPage}
              posts={posts}
              loading={postsLoading}
              error={postsError}
              onCreate={postForm.openCreate}
              onEdit={postForm.openEdit}
              onDelete={postsActions.delete}
              onRefresh={postsActions.refresh}
            />
          ) : (
            <PageManagerEmptyState />
          )}
        </div>
      </div>
      <PageManagerModals pageForm={pageForm} postForm={postForm} />
    </div>
  )
}

export default PageManager
