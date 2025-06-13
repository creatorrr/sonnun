import React, { useState, useCallback, useRef } from 'react'
import EditorPane from './components/EditorPane'
import AssistantPanel from './components/AssistantPanel'
import ProvenanceLegend from './components/ProvenanceLegend'
import './App.css'

interface ProvenanceStats {
  humanPercentage: number
  aiPercentage: number
  citedPercentage: number
  totalCharacters: number
}

const App: React.FC = () => {
  const [provenanceStats, setProvenanceStats] = useState<ProvenanceStats>({
    humanPercentage: 100,
    aiPercentage: 0,
    citedPercentage: 0,
    totalCharacters: 0
  })
  const [isAssistantOpen, setIsAssistantOpen] = useState(true)
  // AIDEV-TODO: Consider using context or state management lib for complex component communication
  const skipEditorUpdateFunctionRef = useRef<(() => void) | null>(null); // Added for Step 3
  
  // AIDEV-NOTE: Component communication - allows AssistantPanel to insert AI content into editor
  const editorRef = useRef<{ insertAIContent: (content: string, model: string) => void } | null>(null)

  // AIDEV-NOTE: State orchestration - manages document content and provenance flow between components
  const handleInsertAIText = useCallback((content: string, model: string) => {
    if (editorRef.current?.insertAIContent) {
      skipEditorUpdateFunctionRef.current?.(); // Call to set skip flag in EditorPane
      editorRef.current.insertAIContent(content, model)
    }
  }, []) // No new dependencies needed for skipEditorUpdateFunctionRef here, it's a ref.

  return (
    <div className="app-container">
      <header className="app-header">
        <h1>Sonnun</h1>
        <div className="header-tools">
          <ProvenanceLegend stats={provenanceStats} className="header-legend" />
        </div>
      </header>
      
      <main className="app-main">
        <EditorPane 
          ref={editorRef}
          onProvenanceChange={setProvenanceStats}
          className="main-editor"
          onReady={(_editor, setSkipFlag) => { // Added onReady for Step 3
            // _editor can be used if App.tsx needs direct editor access, not currently planned.
            // AIDEV-QUESTION: Should we validate setSkipFlag function before storing?
            skipEditorUpdateFunctionRef.current = setSkipFlag;
          }}
        />
        
        <AssistantPanel
          onInsertText={handleInsertAIText}
          isOpen={isAssistantOpen}
          onToggle={() => setIsAssistantOpen(!isAssistantOpen)}
          skipNextUpdate={skipEditorUpdateFunctionRef.current ?? undefined} // Added for Step 3
        />
      </main>
    </div>
  )
}

export default App
