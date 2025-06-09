import React, { useState, useCallback, useRef, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'

interface AssistantPanelProps {
  onInsertText: (text: string, source: string) => void
  isOpen: boolean
  onToggle: () => void
  className?: string
  skipNextUpdate?: () => void // Added skipNextUpdate prop
}

interface Message {
  role: 'user' | 'assistant'
  content: string
  timestamp: Date
  model?: string
  tokenCount?: number
}

interface AIPrompt {
  prompt: string
  model?: string
  max_tokens?: number
}

interface AIResponse {
  content: string
  model: string
  token_count?: number
}

const AssistantPanel: React.FC<AssistantPanelProps> = ({ 
  onInsertText, 
  isOpen, 
  onToggle, 
  className = '',
  skipNextUpdate // Destructure skipNextUpdate
}) => {
  const [messages, setMessages] = useState<Message[]>([])
  const [inputValue, setInputValue] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [selectedModel, setSelectedModel] = useState('gpt-3.5-turbo')
  const [maxTokens, setMaxTokens] = useState(1000)
  const [error, setError] = useState<string | null>(null)
  const messagesEndRef = useRef<HTMLDivElement>(null)

  // AIDEV-NOTE: UX enhancement - maintains chat flow by auto-scrolling to latest message
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  // AIDEV-NOTE: Error state management - clears API errors when user retries input
  useEffect(() => {
    if (inputValue && error) {
      setError(null)
    }
  }, [inputValue, error])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!inputValue.trim() || isLoading) return

    const userMessage: Message = {
      role: 'user',
      content: inputValue.trim(),
      timestamp: new Date()
    }

    setMessages(prev => [...prev, userMessage])
    setInputValue('')
    setIsLoading(true)
    setError(null)

    try {
      // AIDEV-NOTE: Security pattern - API calls go through Rust backend to protect API keys
      const promptData: AIPrompt = {
        prompt: inputValue.trim(),
        model: selectedModel,
        max_tokens: maxTokens
      }

      const response: AIResponse = await invoke('query_ai_assistant', { 
        promptData 
      })

      const assistantMessage: Message = {
        role: 'assistant',
        content: response.content,
        timestamp: new Date(),
        model: response.model,
        tokenCount: response.token_count
      }

      setMessages(prev => [...prev, assistantMessage])
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      setError(errorMessage)
      
      // Add error message to chat
      const errorChatMessage: Message = {
        role: 'assistant',
        content: `Error: ${errorMessage}`,
        timestamp: new Date()
      }
      setMessages(prev => [...prev, errorChatMessage])
    } finally {
      setIsLoading(false)
    }
  }

  // AIDEV-NOTE: Core feature - bridges AI chat to editor with model attribution for provenance
  const handleInsertResponse = useCallback((message: Message) => {
    if (message.role === 'assistant') {
      const model = message.model || selectedModel
      skipNextUpdate?.(); // Call skipNextUpdate before inserting text
      onInsertText(message.content, model)
    }
  }, [onInsertText, selectedModel, skipNextUpdate]) // Added skipNextUpdate to dependencies

  const handleClearChat = useCallback(() => {
    setMessages([])
    setError(null)
  }, [])

  const handleRetry = useCallback(() => {
    if (messages.length > 0) {
      const lastUserMessage = messages.slice().reverse().find(m => m.role === 'user')
      if (lastUserMessage) {
        setInputValue(lastUserMessage.content)
      }
    }
  }, [messages])

  return (
    <div className={`assistant-panel ${isOpen ? 'open' : 'closed'} ${className}`}>
      <div className="assistant-header">
        <h3>AI Assistant</h3>
        <div className="header-actions">
          {isOpen && (
            <button 
              onClick={handleClearChat}
              className="btn-secondary btn-small"
              title="Clear chat history"
            >
              Clear
            </button>
          )}
          <button onClick={onToggle} aria-label="Toggle assistant panel" className="toggle-btn">
            {isOpen ? '×' : '☰'}
          </button>
        </div>
      </div>
      
      {isOpen && (
        <div className="assistant-content">
          {/* Model selection and settings */}
          <div className="assistant-settings">
            <div className="setting-group">
              <label htmlFor="model-select">Model:</label>
              <select 
                id="model-select"
                value={selectedModel}
                onChange={(e) => setSelectedModel(e.target.value)}
                disabled={isLoading}
              >
                <option value="gpt-3.5-turbo">GPT-3.5 Turbo</option>
                <option value="gpt-4">GPT-4</option>
                <option value="gpt-4-turbo">GPT-4 Turbo</option>
              </select>
            </div>
            <div className="setting-group">
              <label htmlFor="max-tokens">Max Tokens:</label>
              <input
                id="max-tokens"
                type="number"
                min="100"
                max="4000"
                value={maxTokens}
                onChange={(e) => setMaxTokens(parseInt(e.target.value) || 1000)}
                disabled={isLoading}
              />
            </div>
          </div>

          {/* Error display */}
          {error && (
            <div className="error-banner">
              <span>{error}</span>
              <button onClick={handleRetry} className="btn-secondary btn-small">
                Retry
              </button>
            </div>
          )}
          
          {/* Messages container */}
          <div className="messages-container">
            {messages.length === 0 && (
              <div className="empty-state">
                <p>Start a conversation with the AI assistant!</p>
                <p className="hint">Try asking: "Help me write about renewable energy"</p>
              </div>
            )}
            
            {messages.map((message, index) => (
              <div key={index} className={`message ${message.role}`}>
                <div className="message-header">
                  <span className="message-role">
                    {message.role === 'user' ? 'You' : (message.model || 'Assistant')}
                  </span>
                  <span className="message-timestamp">
                    {message.timestamp.toLocaleTimeString()}
                  </span>
                  {message.tokenCount && (
                    <span className="message-tokens">{message.tokenCount} tokens</span>
                  )}
                </div>
                <div className="message-content">{message.content}</div>
                {message.role === 'assistant' && (
                  <div className="message-actions">
                    <button
                      onClick={() => handleInsertResponse(message)}
                      className="btn-primary btn-small"
                      title="Insert this response into the document"
                    >
                      Insert into Document
                    </button>
                    <button
                      onClick={() => navigator.clipboard.writeText(message.content)}
                      className="btn-secondary btn-small"
                      title="Copy to clipboard"
                    >
                      Copy
                    </button>
                  </div>
                )}
              </div>
            ))}
            <div ref={messagesEndRef} />
          </div>
          
          {/* Input form */}
          <form onSubmit={handleSubmit} className="input-form">
            <textarea
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              placeholder="Ask the AI assistant..."
              disabled={isLoading}
              rows={3}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault()
                  handleSubmit(e)
                }
              }}
            />
            <div className="form-actions">
              <span className="input-hint">
                Press Enter to send, Shift+Enter for new line
              </span>
              <button type="submit" disabled={isLoading || !inputValue.trim()}>
                {isLoading ? 'Thinking...' : 'Send'}
              </button>
            </div>
          </form>
        </div>
      )}
    </div>
  )
}

export default AssistantPanel