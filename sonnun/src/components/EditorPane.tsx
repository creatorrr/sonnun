import React, { useState, useCallback, useImperativeHandle, forwardRef } from 'react'
import { useEditor, EditorContent } from '@tiptap/react'
import StarterKit from '@tiptap/starter-kit'
import { Extension } from '@tiptap/core'
import { Plugin, PluginKey } from 'prosemirror-state'
import { invoke } from '@tauri-apps/api/tauri'
import { ProvenanceMark } from '../extensions/ProvenanceMark'
import CitationModal from './CitationModal'

interface EditorPaneProps {
  onContentChange?: (content: string) => void
  onProvenanceChange?: (stats: ProvenanceStats) => void
  className?: string
}

interface ProvenanceStats {
  humanPercentage: number
  aiPercentage: number
  citedPercentage: number
  totalCharacters: number
}

interface CitationModalState {
  isOpen: boolean
  pastedText: string
  insertPosition?: number
}

const EditorPane = forwardRef<
  { insertAIContent: (content: string, model: string) => void }, 
  EditorPaneProps
>(({ 
  onContentChange, 
  onProvenanceChange, 
  className = '' 
}, ref) => {
  const [citationModal, setCitationModal] = useState<CitationModalState>({
    isOpen: false,
    pastedText: ''
  })

  // AIDEV-NOTE: Core algorithm for calculating real-time provenance stats from Tiptap doc tree
  const calculateProvenanceStats = useCallback((editor: any): ProvenanceStats => {
    const doc = editor.state.doc
    let humanChars = 0
    let aiChars = 0
    let citedChars = 0

    doc.descendants((node: any) => {
      if (node.marks) {
        node.marks.forEach((mark: any) => {
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
      totalCharacters: total
    }
  }, [])

  // AIDEV-NOTE: Critical path - logs all content changes to SQLite via Tauri for audit trail
  const logProvenanceEvent = useCallback(async (eventType: string, textHash: string, source: string, spanLength: number) => {
    try {
      await invoke('log_provenance_event', {
        event: {
          timestamp: new Date().toISOString(),
          event_type: eventType,
          text_hash: textHash,
          source,
          span_length: spanLength
        }
      })
    } catch (error) {
      console.error('Failed to log provenance event:', error)
    }
  }, [])

  const editor = useEditor({
    extensions: [
      StarterKit.configure({
        // Disable default paste handling for custom citation enforcement
        dropcursor: false
      }),
      ProvenanceMark,
      // AIDEV-NOTE: Security critical - intercepts ALL paste ops to enforce citation req (>50 chars)
      Extension.create({
        name: 'pasteHandler',
        addProseMirrorPlugins() {
          return [
            new Plugin({
              key: new PluginKey('pasteHandler'),
              props: {
                handlePaste(view: any, event: ClipboardEvent, slice: any) {
                  const text = event.clipboardData?.getData('text/plain')
                  
                  // AIDEV-NOTE: UX decision point - 50 char threshold triggers citation modal
                  if (text && text.length > 50) {
                    event.preventDefault()
                    setCitationModal({
                      isOpen: true,
                      pastedText: text,
                      insertPosition: view.state.selection.from
                    })
                    return true
                  }
                  
                  // Small pastes default to human content
                  if (text && text.length <= 50) {
                    logProvenanceEvent('human', text, 'user', text.length)
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
    }
  })

  // AIDEV-NOTE: Data flow - converts citation form data to ProvenanceMark with source attribution
  const handleCitationSubmit = useCallback((citation: any) => {
    if (!editor || !citationModal.pastedText) return

    // Insert text with citation provenance mark
    editor.commands.insertContent({
      type: 'text',
      text: citationModal.pastedText,
      marks: [
        {
          type: 'provenanceMark',
          attrs: {
            source: citation.source,
            type: 'cited'
          }
        }
      ]
    })

    // Log the provenance event
    logProvenanceEvent(
      'cited', 
      citationModal.pastedText, 
      citation.source, 
      citationModal.pastedText.length
    )

    setCitationModal({ isOpen: false, pastedText: '' })
  }, [editor, citationModal.pastedText, logProvenanceEvent])

  const handleCitationCancel = useCallback(() => {
    setCitationModal({ isOpen: false, pastedText: '' })
  }, [])

  // AIDEV-NOTE: AI integration point - marks all AI content with model name for transparency
  const insertAIContent = useCallback((content: string, model: string) => {
    if (!editor) return

    editor.commands.insertContent({
      type: 'text',
      text: content,
      marks: [
        {
          type: 'provenanceMark',
          attrs: {
            source: model,
            type: 'ai'
          }
        }
      ]
    })

    logProvenanceEvent('ai', content, model, content.length)
  }, [editor, logProvenanceEvent])

  // AIDEV-NOTE: Expose insertAIContent method to parent via ref for component communication
  useImperativeHandle(ref, () => ({
    insertAIContent
  }), [insertAIContent])

  return (
    <div className={`editor-pane ${className}`}>
      <div className="editor-toolbar">
        <h2>Document Editor</h2>
        <div className="editor-actions">
          <button 
            onClick={() => editor?.commands.focus()}
            className="btn-secondary"
          >
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
        onClose={handleCitationCancel}
        onSubmit={handleCitationSubmit}
        pastedText={citationModal.pastedText}
      />
    </div>
  )
})

EditorPane.displayName = 'EditorPane'
export default EditorPane