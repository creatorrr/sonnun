import React, { useImperativeHandle, forwardRef, useCallback } from 'react'
import { useEditor, EditorContent, Editor } from '@tiptap/react'
import StarterKit from '@tiptap/starter-kit'
import Link from '@tiptap/extension-link'
import { Extension } from '@tiptap/core'
import { ProvenanceMark } from '../extensions/ProvenanceMark'
import { Plugin, PluginKey } from 'prosemirror-state'
import CitationModal from './CitationModal'
import { useProvenanceTracking } from '../hooks/useProvenanceTracking'
import { useCitationHandler } from '../hooks/useCitationHandler'
import type { ProvenanceStats } from '../utils/manifestGenerator'

interface EditorPaneProps {
  onContentChange?: (content: string) => void
  onProvenanceChange?: (stats: ProvenanceStats) => void
  className?: string
  onReady?: (editor: Editor, setSkipFlag: () => void) => void
}

const EditorPane = forwardRef<
  { insertAIContent: (content: string, model: string) => void },
  EditorPaneProps
>(({ onContentChange, onProvenanceChange, className = '', onReady }, ref) => {
  const editor = useEditor({
    extensions: [
      StarterKit.configure({ dropcursor: false }),
      ProvenanceMark,
      Link,
      Extension.create({
        name: 'pasteHandler',
        addProseMirrorPlugins() {
          return [
            new Plugin({
              key: new PluginKey('pasteHandler'),
              props: {
                handlePaste(view: any, event: ClipboardEvent, _slice: any) {
                  const text = event.clipboardData?.getData('text/plain')
                  if (text && text.length > 10) {
                    event.preventDefault()
                    citation.openCitationModal(text, view.state.selection.from)
                    return true
                  }
                  if (text && text.length <= 10) {
                    provenance.logProvenanceEvent('human', text, 'user', text.length)
                  }
                  return false
                }
              }
            })
          ]
        }
      })
    ],
    content: '<p>Start writing your document...</p>',
    onUpdate: ({ editor }) => {
      const content = editor.getHTML()
      const stats = provenance.calculateProvenanceStats(editor)
      onContentChange?.(content)
      onProvenanceChange?.(stats)
    },
    onTransaction: ({ editor, transaction }) => {
      if (transaction.docChanged) {
        const stats = provenance.calculateProvenanceStats(editor)
        onProvenanceChange?.(stats)
      }
    }
  })

  const provenance = useProvenanceTracking(editor, onReady)
  const citation = useCitationHandler(editor, provenance)

  const insertAIContent = useCallback(
    (content: string, model: string) => {
      if (!editor) return
      editor.commands.insertContent({
        type: 'text',
        text: content,
        marks: [
          {
            type: 'provenanceMark',
            attrs: { source: model, type: 'ai' }
          }
        ]
      })
      provenance.logProvenanceEvent('ai', content, model, content.length)
    },
    [editor, provenance]
  )

  useImperativeHandle(ref, () => ({ insertAIContent }), [insertAIContent])

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
        isOpen={citation.citationModal.isOpen}
        onCancel={citation.handleCitationCancel}
        onConfirm={citation.handleCitationConfirm}
        pastedText={citation.citationModal.pastedText}
      />
    </div>
  )
})

EditorPane.displayName = 'EditorPane'
export default EditorPane
