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
  const [documentContent, setDocumentContent] = useState('')
  const [provenanceStats, setProvenanceStats] = useState<ProvenanceStats>({
    humanPercentage: 100,
    aiPercentage: 0,
    citedPercentage: 0,
    totalCharacters: 0
  })
  const [isAssistantOpen, setIsAssistantOpen] = useState(true)
  
  // AIDEV-NOTE: Component communication - allows AssistantPanel to insert AI content into editor
  const editorRef = useRef<{ insertAIContent: (content: string, model: string) => void } | null>(null)

  // AIDEV-NOTE: State orchestration - manages document content and provenance flow between components
  const handleInsertAIText = useCallback((content: string, model: string) => {
    if (editorRef.current?.insertAIContent) {
      editorRef.current.insertAIContent(content, model)
    }
  }, [])

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
          onContentChange={setDocumentContent}
          onProvenanceChange={setProvenanceStats}
          className="main-editor"
        />
        
        <AssistantPanel
          onInsertText={handleInsertAIText}
          isOpen={isAssistantOpen}
          onToggle={() => setIsAssistantOpen(!isAssistantOpen)}
        />
      </main>
    </div>
  )
}

export default App
