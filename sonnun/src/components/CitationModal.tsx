import React, { useState } from 'react'

interface CitationModalProps {
  isOpen: boolean
  onClose: () => void
  onSubmit: (citation: CitationData) => void
  pastedText: string
}

interface CitationData {
  source: string
  url?: string
  author?: string
  date?: string
}

const CitationModal: React.FC<CitationModalProps> = ({ isOpen, onClose, onSubmit, pastedText }) => {
  const [citation, setCitation] = useState<CitationData>({
    source: '',
    url: '',
    author: '',
    date: new Date().toISOString().split('T')[0],
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (citation.source.trim()) {
      onSubmit(citation)
      // Reset form
      setCitation({
        source: '',
        url: '',
        author: '',
        date: new Date().toISOString().split('T')[0],
      })
    }
  }

  const handleCancel = () => {
    // AIDEV-NOTE: Security decision - canceling citation rejects the paste entirely (no fallback)
    onClose()
  }

  if (!isOpen) return null

  return (
    <div className="modal-overlay" onClick={handleCancel}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <h2>Citation Required</h2>
        <p className="modal-description">
          Please provide source information for the pasted content:
        </p>
        
        <div className="pasted-preview">
          <strong>Pasted text:</strong>
          <p>{pastedText.substring(0, 200)}{pastedText.length > 200 ? '...' : ''}</p>
        </div>

        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label htmlFor="source">
              Source <span className="required">*</span>
            </label>
            <input
              id="source"
              type="text"
              value={citation.source}
              onChange={(e) => setCitation({ ...citation, source: e.target.value })}
              placeholder="e.g., Wikipedia, Research Paper Title"
              required
              autoFocus
            />
          </div>

          <div className="form-group">
            <label htmlFor="url">URL</label>
            <input
              id="url"
              type="url"
              value={citation.url}
              onChange={(e) => setCitation({ ...citation, url: e.target.value })}
              placeholder="https://example.com/article"
            />
          </div>

          <div className="form-group">
            <label htmlFor="author">Author</label>
            <input
              id="author"
              type="text"
              value={citation.author}
              onChange={(e) => setCitation({ ...citation, author: e.target.value })}
              placeholder="John Doe"
            />
          </div>

          <div className="form-group">
            <label htmlFor="date">Date</label>
            <input
              id="date"
              type="date"
              value={citation.date}
              onChange={(e) => setCitation({ ...citation, date: e.target.value })}
            />
          </div>

          <div className="modal-actions">
            <button type="button" onClick={handleCancel} className="btn-secondary">
              Cancel (discard paste)
            </button>
            <button type="submit" className="btn-primary">
              Add Citation
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

export default CitationModal