import { visit } from 'unist-util-visit'

const RE_ALERT = /^\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]/i

export default function remarkGithubAlerts() {
    return (tree) => {
        visit(tree, 'blockquote', (node) => {
            const paragraph = node.children[0]
            if (!paragraph || paragraph.type !== 'paragraph') return

            const text = paragraph.children[0]
            if (!text || text.type !== 'text') return

            const match = text.value.match(RE_ALERT)
            if (match) {
                const type = match[1].toUpperCase()

                // Remove the [!TYPE] marker + optional newline/space
                // Usually it's "[!NOTE]\nContent" or "[!NOTE] Content"
                // Sometimes it might be just "[!NOTE]"

                const contentStart = match[0].length
                let cleanValue = text.value.slice(contentStart)

                if (cleanValue.startsWith('\n')) {
                    cleanValue = cleanValue.slice(1)
                } else if (cleanValue.startsWith(' ')) {
                    cleanValue = cleanValue.slice(1)
                }

                text.value = cleanValue

                // Attach metadata to the blockquote node
                if (!node.data) node.data = {}
                if (!node.data.hProperties) node.data.hProperties = {}

                node.data.hProperties['data-alert-type'] = type
            }
        })
    }
}
