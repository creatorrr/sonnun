import React, {
  useState,
  useCallback,
  useImperativeHandle,
  forwardRef,
  useRef,
  useEffect,
} from 'react'
import { useEditor, EditorContent, Editor } from '@tiptap/react'
import StarterKit from '@tiptap/starter-kit'
import Link from '@tiptap/extension-link'
import { Extension } from '@tiptap/core'
import { ProvenanceMark } from '../extensions/ProvenanceMark'
import { Plugin, PluginKey } from 'prosemirror-state'
import { invoke } from '@tauri-apps/api/core'
import CitationModal from './CitationModal'
import { diffChars } from 'diff'
import type { ProvenanceStats } from '../utils/manifestGenerator'

interface EditorPaneProps {
  onContentChange?: (content: string) => void
  onProvenanceChange?: (stats: ProvenanceStats) => void
  className?: string
  onReady?: (editor: Editor, setSkipFlag: () => void) => void
}

// Helper function for Step 3
// AIDEV-NOTE: perf-sensitive - diff new vs old to detect human input
const findInsertedText = (oldText: string, newText: string): string => {
  const diff = diffChars(oldText, newText)
  let inserted = ''
  diff.forEach((part) => {
    if (part.added) inserted += part.value
  })
  return inserted
}

interface CitationModalState {
  isOpen: boolean
  pastedText: string
  insertPosition?: number
}

const EditorPane = forwardRef<
  { insertAIContent: (content: string, model: string) => void },
  EditorPaneProps
>(
  (
    {
      onContentChange,
      onProvenanceChange,
      className = '',
      onReady, // Destructure onReady
    },
    ref
  ) => {
    // AIDEV-NOTE: Counter handles overlapping async insertions
    const lastContentRef = useRef<string>('')
    const skipUpdateCountRef = useRef<number>(0)

    const [citationModal, setCitationModal] = useState<CitationModalState>({
      isOpen: false,
      pastedText: '',
    })

    // AIDEV-NOTE: Core algorithm for calculating real-time provenance stats from Tiptap doc tree
    // AIDEV-QUESTION: Should we cache stats calculation for large documents (>10k chars)?
    const calculateProvenanceStats = useCallback((editor: any): ProvenanceStats => {
      const doc = editor.state.doc
      let humanChars = 0
      let aiChars = 0
      let citedChars = 0

      // AIDEV-TODO: Add proper TypeScript types for node parameter
      doc.descendants((node: any) => {
        if (node.marks) {
          node.marks.forEach((mark: any) => {
            // Reverted to 'provenanceMark' to match the existing extension's name
            if (mark.type.name === 'provenanceMark') {
              const text = node.textContent
              switch (mark.attrs.type) {
                case 'human':
                  humanChars += text.length
                  break
                case 'ai':
                  aiChars += text.length
                  break
                case 'cited':
                  citedChars += text.length
                  break
              }
            }
          })
        } else if (node.isText) {
          // Text without provenance marks defaults to human
          humanChars += node.textContent.length
        }
      })

      const total = humanChars + aiChars + citedChars
      return {
        humanPercentage: total > 0 ? (humanChars / total) * 100 : 0,
        aiPercentage: total > 0 ? (aiChars / total) * 100 : 0,
        citedPercentage: total > 0 ? (citedChars / total) * 100 : 0,
        totalCharacters: total,
      }
    }, [])

    // AIDEV-NOTE: Critical path - logs all content changes to SQLite via Tauri for audit trail
    const logProvenanceEvent = useCallback(
      async (eventType: string, text: string, source: string, spanLength: number) => {
        try {
          await invoke('log_provenance_event', {
            event: {
              timestamp: new Date().toISOString(),
              event_type: eventType,
              text,
              source,
              span_length: spanLength,
            },
          })
        } catch (error) {
          console.error('Failed to log provenance event:', error)
        }
      },
      []
    )

    const editor = useEditor({
      extensions: [
        StarterKit.configure({
          // Disable default paste handling for custom citation enforcement
          dropcursor: false,
        }),
        ProvenanceMark,
        Link,
        // AIDEV-NOTE: Security critical - intercepts ALL paste ops to enforce citation req (>50 chars)
        Extension.create({
          name: 'pasteHandler',
          addProseMirrorPlugins() {
            return [
              new Plugin({
                key: new PluginKey('pasteHandler'),
                props: {
                  handlePaste(view: any, event: ClipboardEvent, _slice: any) {
                    const text = event.clipboardData?.getData('text/plain')

                    // AIDEV-NOTE: UX decision point - 50 char threshold triggers citation modal
                    // AIDEV-QUESTION: Should threshold be configurable? Current is 10 chars
                    if (text && text.length > 10) {
                      event.preventDefault()
                      setCitationModal({
                        isOpen: true,
                        pastedText: text,
                        insertPosition: view.state.selection.from,
                      })
                      return true
                    }

                    // Small pastes default to human content
                    if (text && text.length <= 10) {
                      logProvenanceEvent('human', text, 'user', text.length)
                    }

                    return false
                  },
                },
              }),
            ]
          },
        }),
      ],
      content: '<p>Start writing your document...</p>',
      onUpdate: ({ editor }) => {
        const content = editor.getHTML()
        const stats = calculateProvenanceStats(editor)

        onContentChange?.(content)
        onProvenanceChange?.(stats)
      },
      onTransaction: ({ editor, transaction }) => {
        // AIDEV-NOTE: Performance sensitive - fires on every keystroke, batched for efficiency
        if (transaction.docChanged) {
          const stats = calculateProvenanceStats(editor)
          onProvenanceChange?.(stats)
        }
      },
    })

    // Plan Step 2: Create handleCitationConfirm function (renamed from handleCitationSubmit):
    useEffect(() => {
      if (editor) {
        // Initialize lastContentRef with current editor text
        lastContentRef.current = editor.getText()

        // AIDEV-TODO: Add cleanup to prevent memory leaks on component unmount

        const handleEditorUpdate = () => {
          if (!editor) return

          // AIDEV-NOTE: Race condition risk - multiple async ops can interfere
          if (skipUpdateCountRef.current > 0) {
            skipUpdateCountRef.current -= 1
            lastContentRef.current = editor.getText()
            // console.log('Skipped update for logging human input');
            return
          }

          const currentText = editor.getText()
          const lastText = lastContentRef.current

          if (currentText.length > lastText.length) {
            const insertedText = findInsertedText(lastText, currentText)
            if (insertedText.trim().length > 0) {
              // console.log('Human typed:', insertedText);
              logProvenanceEvent('human', insertedText, 'user', insertedText.length)
            }
          }
          lastContentRef.current = currentText
        }

        editor.on('update', handleEditorUpdate)

        // Call onReady if provided
        if (onReady) {
          onReady(editor, () => {
            skipUpdateCountRef.current += 1
            // console.log('Skip next update flag set via onReady');
          })
        }

        return () => {
          editor.off('update', handleEditorUpdate)
        }
      }
    }, [editor, onReady, logProvenanceEvent]) // Added dependencies

    const handleCitationConfirm = useCallback(
      (citation: string) => {
        // Signature changed to citation: string
        if (!editor || !citationModal.pastedText) return

        skipUpdateCountRef.current += 1 // Skip logging for this programmatic insertion

        const { pastedText, insertPosition } = citationModal
        const currentPosition = editor.state.selection.from
        const resolvedInsertPos = insertPosition !== undefined ? insertPosition : currentPosition

        // Insert content with provenance mark
        // For compatibility with existing `calculateProvenanceStats`, we ensure `type: 'cited'` is set.
        editor
          .chain()
          .focus()
          .insertContentAt(resolvedInsertPos, pastedText, {
            marks: [
              {
                type: editor.schema.marks.provenanceMark ? 'provenanceMark' : ProvenanceMark.name,
                attrs: { source: citation, type: 'cited' },
              },
            ],
          })
          .run()

        const isURL = /^(ftp|http|https):\/\/[^ "]+$/.test(citation)
        const insertedTextStartPos = resolvedInsertPos
        const insertedTextEndPos = resolvedInsertPos + pastedText.length

        if (isURL) {
          editor
            .chain()
            .focus()
            .setTextSelection({ from: insertedTextStartPos, to: insertedTextEndPos })
            .setLink({ href: citation })
            // Move cursor to the end of the linked text
            .setTextSelection({ from: insertedTextEndPos, to: insertedTextEndPos })
            .run()
        } else {
          // If not a URL, insert ` (source: \${citation})` after the pasted content.
          editor.chain().focus().insertContentAt(insertedTextEndPos, ` (source: ${citation})`).run()
        }

        logProvenanceEvent('cited', pastedText, citation, pastedText.length)

        setCitationModal({ isOpen: false, pastedText: '', insertPosition: undefined })
      },
      [editor, citationModal, logProvenanceEvent, ProvenanceMark.name]
    )

    // Plan Step 2: Create handleCitationCancel function:
    const handleCitationCancel = useCallback(() => {
      setCitationModal({ isOpen: false, pastedText: '', insertPosition: undefined })
    }, [])

    // AIDEV-NOTE: AI integration point - marks all AI content with model name for transparency
    const insertAIContent = useCallback(
      (content: string, model: string) => {
        if (!editor) return

        editor.commands.insertContent({
          type: 'text',
          text: content,
          marks: [
            {
              type: 'provenanceMark',
              attrs: {
                source: model,
                type: 'ai',
              },
            },
          ],
        })

        logProvenanceEvent('ai', content, model, content.length)
      },
      [editor, logProvenanceEvent]
    )

    // AIDEV-NOTE: Expose insertAIContent method to parent via ref for component communication
    useImperativeHandle(
      ref,
      () => ({
        insertAIContent,
      }),
      [insertAIContent]
    )

    return (
      <div className={`editor-pane ${className}`}>
        <div className="editor-toolbar">
          <h2>Document Editor</h2>
          <div className="editor-actions">
            <button onClick={() => editor?.commands.focus()} className="btn-secondary">
              Focus Editor
            </button>
            <button
              onClick={() => {
                const content = editor?.getHTML() || ''
                navigator.clipboard.writeText(content)
              }}
              className="btn-secondary"
            >
              Copy HTML
            </button>
          </div>
        </div>

        <div className="editor-content">
          <EditorContent editor={editor} />
        </div>

        <CitationModal
          isOpen={citationModal.isOpen}
          onCancel={handleCitationCancel}
          onConfirm={handleCitationConfirm}
          pastedText={citationModal.pastedText}
        />
      </div>
    )
  }
)

EditorPane.displayName = 'EditorPane'
export default EditorPane
