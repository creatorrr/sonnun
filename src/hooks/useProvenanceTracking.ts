import { useCallback, useEffect, useRef } from 'react'
import type { Editor } from '@tiptap/react'
import { invoke } from '@tauri-apps/api/core'
import type { ProvenanceStats } from '../utils/manifestGenerator'

// Helper used to determine inserted text between two editor snapshots
const findInsertedText = (oldText: string, newText: string): string => {
  let start = 0
  while (start < oldText.length && start < newText.length && oldText[start] === newText[start]) {
    start++
  }
  let oldEnd = oldText.length - 1
  let newEnd = newText.length - 1
  while (oldEnd >= start && newEnd >= start && oldText[oldEnd] === newText[newEnd]) {
    oldEnd--
    newEnd--
  }
  return newText.slice(start, newEnd + 1)
}

export function useProvenanceTracking(
  editor: Editor | null,
  onReady?: (editor: Editor, setSkipFlag: () => void) => void
) {
  const lastContentRef = useRef<string>('')
  const skipNextUpdateRef = useRef<boolean>(false)

  const setSkipFlag = useCallback(() => {
    skipNextUpdateRef.current = true
  }, [])

  const calculateProvenanceStats = useCallback((ed: Editor): ProvenanceStats => {
    const doc = ed.state.doc
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

  const logProvenanceEvent = useCallback(
    async (eventType: string, textHash: string, source: string, spanLength: number) => {
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
    },
    []
  )

  useEffect(() => {
    if (!editor) return

    lastContentRef.current = editor.getText()

    const handleEditorUpdate = () => {
      if (!editor) return

      if (skipNextUpdateRef.current) {
        skipNextUpdateRef.current = false
        lastContentRef.current = editor.getText()
        return
      }

      const currentText = editor.getText()
      const lastText = lastContentRef.current

      if (currentText.length > lastText.length) {
        const insertedText = findInsertedText(lastText, currentText)
        if (insertedText.trim().length > 0) {
          logProvenanceEvent('human', insertedText, 'user', insertedText.length)
        }
      }
      lastContentRef.current = currentText
    }

    editor.on('update', handleEditorUpdate)

    if (onReady) {
      onReady(editor, setSkipFlag)
    }

    return () => {
      editor.off('update', handleEditorUpdate)
    }
  }, [editor, onReady, logProvenanceEvent, setSkipFlag])

  return { calculateProvenanceStats, logProvenanceEvent, setSkipFlag }
}

export type UseProvenanceTrackingReturn = ReturnType<typeof useProvenanceTracking>
