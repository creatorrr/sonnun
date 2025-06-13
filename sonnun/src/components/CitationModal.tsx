import React, { useState } from 'react'

interface CitationModalProps {
  isOpen: boolean
  pastedText: string
  onConfirm: (citation: string) => void
  onCancel: () => void
}

// AIDEV-NOTE: Simple modal for citation enforcement - replaced complex multi-field form
const CitationModal: React.FC<CitationModalProps> = ({
  isOpen,
  pastedText,
  onConfirm,
  onCancel
}) => {
  const [citation, setCitation] = useState('')

  if (!isOpen) return null

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    // AIDEV-TODO: Add URL validation before confirming
    if (citation.trim()) {
      onConfirm(citation.trim())
      setCitation('')
    }
  }

  return (
    <div className="modal-overlay">
      <div className="modal-content">
        <h3>Citation Required</h3>
        <p>You pasted content that needs a source citation:</p>
        {/* AIDEV-NOTE: Preview limited to 200 chars for UI consistency */}
        <div className="pasted-preview">
          {pastedText.substring(0, 200)}
          {pastedText.length > 200 && '...'}
        </div>

        <form onSubmit={handleSubmit}>
          {/* AIDEV-TODO: Add accessibility labels and ARIA attributes */}
          <input
            type="text"
            value={citation}
            onChange={(e) => setCitation(e.target.value)}
            placeholder="Enter URL or source description..."
            className="citation-input"
            autoFocus
          />
          <div className="modal-actions">
            <button type="button" onClick={onCancel} className="cancel-btn">
              Cancel
            </button>
            <button type="submit" disabled={!citation.trim()} className="confirm-btn">
              Add Citation
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

export default CitationModal
