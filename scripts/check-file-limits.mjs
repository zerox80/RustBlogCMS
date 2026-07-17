import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'

const MAX_LINES = 500
const ROOT = process.cwd()
const EXTENSIONS = new Set(['.cjs', '.css', '.js', '.jsx', '.mjs', '.rs'])
const IGNORED_DIRECTORIES = new Set(['.git', 'dist', 'node_modules', 'target'])

const sourceFiles = []

const collectFiles = (directory) => {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    if (entry.isDirectory() && IGNORED_DIRECTORIES.has(entry.name)) continue

    const absolutePath = path.join(directory, entry.name)
    if (entry.isDirectory()) collectFiles(absolutePath)
    if (entry.isFile() && EXTENSIONS.has(path.extname(entry.name))) sourceFiles.push(absolutePath)
  }
}

collectFiles(ROOT)

const violations = []

for (const file of sourceFiles) {
  const lines = fs.readFileSync(file, 'utf8').split(/\r?\n/)
  if (lines.at(-1) === '') lines.pop()

  if (lines.length > MAX_LINES) {
    violations.push(`${path.relative(ROOT, file)}: ${lines.length} lines (maximum ${MAX_LINES})`)
  }
}

if (violations.length > 0) {
  console.error(`Source file limits failed with ${violations.length} violation(s):`)
  violations.forEach((violation) => console.error(`- ${violation}`))
  process.exitCode = 1
} else {
  console.log(
    `Checked ${sourceFiles.length} Rust, JavaScript, and CSS files: all file line limits passed.`,
  )
}
