import React, { useState, useCallback } from 'react'
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
  const [isAssistantOpen, setIsAssistantOpen] = useState(false)

  // AIDEV-NOTE: State orchestration - manages document content and provenance flow between components
  const handleInsertAIText = useCallback((content: string, model: string) => {
    // This will be handled by EditorPane's insertAIContent method
    console.log('AI content to insert:', { content, model })
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
