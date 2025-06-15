import { useCallback, useState } from 'react'
import type { Editor } from '@tiptap/react'
import { ProvenanceMark } from '../extensions/ProvenanceMark'
import type { UseProvenanceTrackingReturn } from './useProvenanceTracking'

interface CitationModalState {
  isOpen: boolean
  pastedText: string
  insertPosition?: number
}

export function useCitationHandler(
  editor: Editor | null,
  provenance: UseProvenanceTrackingReturn
) {
  const [citationModal, setCitationModal] = useState<CitationModalState>({
    isOpen: false,
    pastedText: ''
  })

  const openCitationModal = useCallback((text: string, position: number) => {
    setCitationModal({ isOpen: true, pastedText: text, insertPosition: position })
  }, [])

  const handleCitationConfirm = useCallback(
    (citation: string) => {
      if (!editor || !citationModal.pastedText) return

      provenance.setSkipFlag()

      const { pastedText, insertPosition } = citationModal
      const currentPosition = editor.state.selection.from
      const resolvedInsertPos = insertPosition !== undefined ? insertPosition : currentPosition

      editor
        .chain()
        .focus()
        .insertContentAt(resolvedInsertPos, pastedText, {
          marks: [
            {
              type: editor.schema.marks.provenanceMark ? 'provenanceMark' : ProvenanceMark.name,
              attrs: { source: citation, type: 'cited' }
            }
          ]
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
          .setTextSelection({ from: insertedTextEndPos, to: insertedTextEndPos })
          .run()
      } else {
        editor.chain().focus().insertContentAt(insertedTextEndPos, ` (source: ${citation})`).run()
      }

      provenance.logProvenanceEvent('cited', pastedText, citation, pastedText.length)

      setCitationModal({ isOpen: false, pastedText: '', insertPosition: undefined })
    },
    [editor, citationModal, provenance]
  )

  const handleCitationCancel = useCallback(() => {
    setCitationModal({ isOpen: false, pastedText: '', insertPosition: undefined })
  }, [])

  return { citationModal, openCitationModal, handleCitationConfirm, handleCitationCancel }
}

export type UseCitationHandlerReturn = ReturnType<typeof useCitationHandler>
