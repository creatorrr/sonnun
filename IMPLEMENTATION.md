# Sonnun Implementation Plan: Step-by-Step Guide

## Overview
This plan breaks down Sonnun development into 6 phases, each building incrementally on the previous. Each step includes specific files to create, commands to run, and testing criteria.

**Total estimated time**: 12-16 days for MVP + signing
**Prerequisites**: Node.js, Rust, OpenAI API key

---

## Phase 1: Foundation & Basic Editor (Days 1-2)

### Step 1.1: Initialize Tauri Project (30 min)
```bash
# Create new Tauri project
npm create tauri-app@latest sonnun --template react-ts
cd sonnun
npm install

# Test basic setup
npm run tauri dev
```
**Verify**: Tauri window opens with default React content
**Files created**: Complete Tauri project structure

### Step 1.2: Install Core Dependencies (15 min)
```bash
# Frontend dependencies
npm install @tiptap/react @tiptap/starter-kit @tiptap/extension-link @tiptap/core
npm install @tauri-apps/api @tauri-apps/plugin-sql

# Add to src-tauri/Cargo.toml
[dependencies]
tauri = { version = "2.0", features = ["shell-open"] }
tauri-plugin-sql = { version = "2.0", features = ["sqlite"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sha2 = "0.10"
chrono = { version = "0.4", features = ["serde"] }
rusqlite = "0.29"
```
**Verify**: `npm run tauri dev` still works without errors

### Step 1.3: Create Basic Layout Structure (45 min)
**Create**: `src/App.tsx`
```tsx
import React, { useState, useRef } from 'react'
import './App.css'

function App() {
  return (
    <div className="app-container">
      <div className="editor-pane">
        <h2>Editor</h2>
        <div className="editor-placeholder">Tiptap editor will go here</div>
      </div>
      <div className="assistant-pane">
        <h2>AI Assistant</h2>
        <div className="assistant-placeholder">ChatGPT integration will go here</div>
      </div>
      <div className="toolbar">
        <button className="publish-btn">Publish</button>
      </div>
    </div>
  )
}

export default App
```

**Create**: `src/App.css`
```css
.app-container {
  display: flex;
  height: 100vh;
  font-family: system-ui, -apple-system, sans-serif;
}

.editor-pane {
  flex: 1;
  padding: 20px;
  border-right: 1px solid #e1e5e9;
}

.assistant-pane {
  width: 350px;
  padding: 20px;
  background: #f8f9fa;
}

.toolbar {
  position: absolute;
  top: 20px;
  right: 20px;
}

.publish-btn {
  background: #0969da;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
}

.editor-placeholder, .assistant-placeholder {
  border: 2px dashed #d1d9e0;
  padding: 40px;
  text-align: center;
  color: #656d76;
  border-radius: 8px;
}
```
**Verify**: Layout shows two-pane design with placeholder content

### Step 1.4: Integrate Basic Tiptap Editor (60 min)
**Create**: `src/components/EditorPane.tsx`
```tsx
import React, { useEffect } from 'react'
import { useEditor, EditorContent } from '@tiptap/react'
import StarterKit from '@tiptap/starter-kit'

interface EditorPaneProps {
  onReady?: (editor: any) => void
}

const EditorPane: React.FC<EditorPaneProps> = ({ onReady }) => {
  const editor = useEditor({
    extensions: [StarterKit],
    content: '<p>Start writing your article...</p>',
    editorProps: {
      attributes: {
        class: 'prose prose-sm sm:prose lg:prose-lg xl:prose-2xl mx-auto focus:outline-none',
      },
    },
  })

  useEffect(() => {
    if (editor && onReady) {
      onReady(editor)
    }
  }, [editor, onReady])

  return (
    <div className="editor-container">
      <EditorContent editor={editor} />
    </div>
  )
}

export default EditorPane
```

**Update**: `src/App.tsx`
```tsx
import React, { useState, useRef } from 'react'
import EditorPane from './components/EditorPane'
import './App.css'

function App() {
  const [editorInstance, setEditorInstance] = useState(null)

  return (
    <div className="app-container">
      <div className="editor-pane">
        <h2>Editor</h2>
        <EditorPane onReady={setEditorInstance} />
      </div>
      <div className="assistant-pane">
        <h2>AI Assistant</h2>
        <div className="assistant-placeholder">ChatGPT integration will go here</div>
      </div>
      <div className="toolbar">
        <button className="publish-btn">Publish</button>
      </div>
    </div>
  )
}

export default App
```

**Create**: `src/components/` directory
**Verify**: Rich text editor works (bold, italic, headings, lists)

### Step 1.5: Add Basic Styling (30 min)
**Update**: `src/App.css` with editor styles
```css
/* Add to existing CSS */
.editor-container {
  min-height: 400px;
  border: 1px solid #e1e5e9;
  border-radius: 8px;
  padding: 16px;
}

.ProseMirror {
  outline: none;
  min-height: 300px;
}

.ProseMirror p {
  margin: 16px 0;
}

.ProseMirror h1, .ProseMirror h2, .ProseMirror h3 {
  margin: 24px 0 16px 0;
  font-weight: 600;
}
```
**Verify**: Editor has clean, readable styling

---

## Phase 2: AI Integration & Event Logging (Days 3-4)

### Step 2.1: Set Up Rust Backend Structure (45 min)
**Update**: `src-tauri/src/main.rs`
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct PromptRequest {
    prompt: String,
}

#[derive(Debug, Serialize)]
struct ApiResponse {
    success: bool,
    data: Option<String>,
    error: Option<String>,
}

#[tauri::command]
async fn test_command() -> Result<String, String> {
    Ok("Backend is working!".to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![test_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Test backend connection**:
**Update**: `src/App.tsx` to test Rust connection
```tsx
import { invoke } from '@tauri-apps/api/tauri'

// Add to App component
const testBackend = async () => {
  try {
    const result = await invoke('test_command')
    console.log('Backend test:', result)
  } catch (error) {
    console.error('Backend error:', error)
  }
}

// Add button to UI temporarily
<button onClick={testBackend}>Test Backend</button>
```
**Verify**: Console shows "Backend is working!" when button clicked

### Step 2.2: Implement OpenAI Integration (90 min)
**Update**: `src-tauri/src/main.rs` with OpenAI command
```rust
use reqwest::Client;
use serde_json::Value;
use std::env;

#[tauri::command]
async fn openai_complete(prompt: String) -> Result<String, String> {
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| "OpenAI API key not found in environment")?;
    
    let client = Client::new();
    let payload = serde_json::json!({
        "model": "gpt-3.5-turbo",
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": 500,
        "temperature": 0.7
    });
    
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&api_key)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("API returned status: {}", response.status()));
    }
    
    let json: Value = response.json().await
        .map_err(|e| format!("JSON parse error: {}", e))?;
    
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in response")?
        .to_string();
    
    Ok(content)
}

// Update main function
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![test_command, openai_complete])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Create test**: Add environment variable and test
```bash
export OPENAI_API_KEY="your-key-here"
npm run tauri dev
```

**Test OpenAI in frontend**:
```tsx
const testOpenAI = async () => {
  try {
    const result = await invoke('openai_complete', { prompt: 'Say hello!' })
    console.log('OpenAI response:', result)
  } catch (error) {
    console.error('OpenAI error:', error)
  }
}
```
**Verify**: OpenAI returns generated text

### Step 2.3: Set Up SQLite Database (60 min)
**Update**: `src-tauri/Cargo.toml` - ensure SQL plugin is configured
**Update**: `src-tauri/tauri.conf.json` - add SQL plugin permissions

**Update**: `src-tauri/src/main.rs` with database setup
```rust
use tauri_plugin_sql::{Builder as SqlBuilder, Migration, MigrationKind};

fn create_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "create events table",
        sql: "
            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                event_type TEXT NOT NULL,
                text_hash TEXT NOT NULL,
                source TEXT,
                span_length INTEGER,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
        kind: MigrationKind::Up,
    }]
}

#[tauri::command]
fn log_event(event_type: String, text: String, source: String) -> Result<(), String> {
    use sha2::{Sha256, Digest};
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let hash = format!("{:x}", Sha256::digest(text.as_bytes()));
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();
    
    let conn = rusqlite::Connection::open("sonnun.db")
        .map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT INTO events (timestamp, event_type, text_hash, source, span_length) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![timestamp, event_type, hash, source, text.len()]
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

// Update main function
fn main() {
    tauri::Builder::default()
        .plugin(SqlBuilder::default()
            .add_migrations("sqlite:sonnun.db", create_migrations())
            .build())
        .invoke_handler(tauri::generate_handler![
            test_command, 
            openai_complete, 
            log_event
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Test database**:
```tsx
const testDatabase = async () => {
  try {
    await invoke('log_event', {
      eventType: 'test',
      text: 'Hello world',
      source: 'manual'
    })
    console.log('Database log successful')
  } catch (error) {
    console.error('Database error:', error)
  }
}
```
**Verify**: No errors, `sonnun.db` file created

### Step 2.4: Build AI Assistant Panel (75 min)
**Create**: `src/components/AssistantPanel.tsx`
```tsx
import React, { useState } from 'react'
import { invoke } from '@tauri-apps/api/tauri'

interface AssistantPanelProps {
  onInsertText?: (text: string) => void
}

const AssistantPanel: React.FC<AssistantPanelProps> = ({ onInsertText }) => {
  const [prompt, setPrompt] = useState('')
  const [loading, setLoading] = useState(false)
  const [lastResponse, setLastResponse] = useState('')

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!prompt.trim() || loading) return

    setLoading(true)
    try {
      const response = await invoke<string>('openai_complete', { prompt })
      setLastResponse(response)
      setPrompt('')
      
      // Log the AI generation event
      await invoke('log_event', {
        eventType: 'ai',
        text: response,
        source: 'gpt-3.5-turbo'
      })
      
    } catch (error) {
      console.error('AI request failed:', error)
      setLastResponse('Error: ' + error)
    } finally {
      setLoading(false)
    }
  }

  const insertIntoEditor = () => {
    if (lastResponse && onInsertText) {
      onInsertText(lastResponse)
    }
  }

  return (
    <div className="assistant-panel">
      <form onSubmit={handleSubmit} className="prompt-form">
        <textarea
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          placeholder="Ask AI to help with your writing..."
          className="prompt-input"
          rows={3}
          disabled={loading}
        />
        <button 
          type="submit" 
          disabled={!prompt.trim() || loading}
          className="ask-button"
        >
          {loading ? 'Generating...' : 'Ask AI'}
        </button>
      </form>
      
      {lastResponse && (
        <div className="response-container">
          <div className="response-text">{lastResponse}</div>
          <button onClick={insertIntoEditor} className="insert-button">
            Insert into Editor
          </button>
        </div>
      )}
    </div>
  )
}

export default AssistantPanel
```

**Update**: `src/App.css` with assistant styles
```css
.assistant-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.prompt-form {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.prompt-input {
  resize: vertical;
  padding: 12px;
  border: 1px solid #e1e5e9;
  border-radius: 6px;
  font-family: inherit;
}

.ask-button {
  background: #28a745;
  color: white;
  border: none;
  padding: 10px;
  border-radius: 6px;
  cursor: pointer;
}

.ask-button:disabled {
  background: #6c757d;
  cursor: not-allowed;
}

.response-container {
  border: 1px solid #e1e5e9;
  border-radius: 6px;
  padding: 12px;
  background: white;
}

.response-text {
  margin-bottom: 8px;
  line-height: 1.5;
}

.insert-button {
  background: #007bff;
  color: white;
  border: none;
  padding: 6px 12px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
```

**Update**: `src/App.tsx` to use AssistantPanel
```tsx
import AssistantPanel from './components/AssistantPanel'

// In App component
const handleInsertText = (text: string) => {
  if (editorInstance) {
    editorInstance.commands.insertContent(text)
  }
}

// In JSX
<div className="assistant-pane">
  <h2>AI Assistant</h2>
  <AssistantPanel onInsertText={handleInsertText} />
</div>
```

**Verify**: AI assistant generates text and inserts into editor

---

## Phase 3: Provenance Tracking & Citation (Days 5-6)

### Step 3.1: Create Provenance Mark Extension (60 min)
**Create**: `src/extensions/ProvenanceMark.ts`
```typescript
import { Mark } from '@tiptap/core'

export interface ProvenanceOptions {
  HTMLAttributes: Record<string, any>
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    provenance: {
      setProvenance: (attributes: { source: string }) => ReturnType
      unsetProvenance: () => ReturnType
    }
  }
}

export const ProvenanceMark = Mark.create<ProvenanceOptions>({
  name: 'provenance',

  addOptions() {
    return {
      HTMLAttributes: {},
    }
  },

  addAttributes() {
    return {
      source: {
        default: null,
        parseHTML: element => element.getAttribute('data-provenance'),
        renderHTML: attributes => {
          if (!attributes.source) return {}
          return { 'data-provenance': attributes.source }
        },
      },
    }
  },

  parseHTML() {
    return [
      {
        tag: 'span[data-provenance]',
      },
    ]
  },

  renderHTML({ mark, HTMLAttributes }) {
    return ['span', { ...HTMLAttributes, 'data-provenance': mark.attrs.source }, 0]
  },

  addCommands() {
    return {
      setProvenance: attributes => ({ commands }) => {
        return commands.setMark(this.name, attributes)
      },
      unsetProvenance: () => ({ commands }) => {
        return commands.unsetMark(this.name)
      },
    }
  },

  // Don't extend the mark to new content typed after
  inclusive() {
    return false
  },
})
```

**Update**: `src/components/EditorPane.tsx` to use provenance mark
```tsx
import Link from '@tiptap/extension-link'
import { ProvenanceMark } from '../extensions/ProvenanceMark'

const editor = useEditor({
  extensions: [
    StarterKit,
    Link,
    ProvenanceMark
  ],
  // ... rest of config
})
```

**Verify**: Editor loads without errors, inspect HTML to see data attributes

### Step 3.2: Implement Paste Interception (90 min)
**Create**: `src/components/CitationModal.tsx`
```tsx
import React, { useState } from 'react'

interface CitationModalProps {
  isOpen: boolean
  pastedText: string
  onConfirm: (citation: string) => void
  onCancel: () => void
}

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
        <div className="pasted-preview">
          {pastedText.substring(0, 200)}
          {pastedText.length > 200 && '...'}
        </div>
        
        <form onSubmit={handleSubmit}>
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
```

**Update**: `src/App.css` with modal styles
```css
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: white;
  padding: 24px;
  border-radius: 8px;
  width: 500px;
  max-width: 90vw;
}

.pasted-preview {
  background: #f8f9fa;
  padding: 12px;
  border-radius: 4px;
  margin: 12px 0;
  font-family: monospace;
  font-size: 14px;
  max-height: 100px;
  overflow-y: auto;
}

.citation-input {
  width: 100%;
  padding: 12px;
  border: 1px solid #e1e5e9;
  border-radius: 6px;
  margin: 12px 0;
}

.modal-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.cancel-btn {
  background: #6c757d;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
}

.confirm-btn {
  background: #007bff;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
}

.confirm-btn:disabled {
  background: #6c757d;
  cursor: not-allowed;
}
```

**Update**: `src/components/EditorPane.tsx` with paste handling
```tsx
import { useState } from 'react'
import CitationModal from './CitationModal'
import { invoke } from '@tauri-apps/api/tauri'

const EditorPane: React.FC<EditorPaneProps> = ({ onReady }) => {
  const [showCitationModal, setShowCitationModal] = useState(false)
  const [pendingPaste, setPendingPaste] = useState('')

  const editor = useEditor({
    extensions: [StarterKit, Link, ProvenanceMark],
    content: '<p>Start writing your article...</p>',
    editorProps: {
      attributes: {
        class: 'prose prose-sm sm:prose lg:prose-lg xl:prose-2xl mx-auto focus:outline-none',
      },
      handlePaste(view, event) {
        const text = event.clipboardData?.getData('text/plain')
        if (text && text.length > 10) { // Only require citation for substantial pastes
          event.preventDefault()
          setPendingPaste(text)
          setShowCitationModal(true)
          return true
        }
        return false // Allow normal paste for short text
      },
    },
  })

  const handleCitationConfirm = async (citation: string) => {
    if (!editor || !pendingPaste) return

    const isURL = citation.startsWith('http://') || citation.startsWith('https://')
    
    // Insert the text with provenance marking
    editor.chain().focus()
      .setProvenance({ source: 'cited' })
      .insertContent(pendingPaste)
      .unsetProvenance()
      .run()

    // If it's a URL, make the text a link
    if (isURL) {
      const { from, to } = editor.state.selection
      editor.commands.setTextSelection({
        from: from - pendingPaste.length,
        to: from
      })
      editor.commands.setLink({ href: citation })
      editor.commands.setTextSelection(to)
    } else {
      // Add source note in parentheses
      editor.commands.insertContent(` (source: ${citation})`)
    }

    // Log the citation event
    await invoke('log_event', {
      eventType: 'cited',
      text: pendingPaste,
      source: citation
    })

    setShowCitationModal(false)
    setPendingPaste('')
  }

  const handleCitationCancel = () => {
    setShowCitationModal(false)
    setPendingPaste('')
  }

  return (
    <div className="editor-container">
      <EditorContent editor={editor} />
      <CitationModal
        isOpen={showCitationModal}
        pastedText={pendingPaste}
        onConfirm={handleCitationConfirm}
        onCancel={handleCitationCancel}
      />
    </div>
  )
}
```

**Verify**: Pasting text >10 chars opens citation modal, adds provenance marks

### Step 3.3: Add Human Typing Detection (75 min)
**Update**: `src/components/EditorPane.tsx` with typing detection
```tsx
import { useEffect, useRef } from 'react'

const EditorPane: React.FC<EditorPaneProps> = ({ onReady }) => {
  const lastContentRef = useRef('')
  const skipNextUpdateRef = useRef(false)

  useEffect(() => {
    if (!editor) return

    const handleUpdate = ({ editor: currentEditor }) => {
      // Skip logging if this was a programmatic insertion (AI or citation)
      if (skipNextUpdateRef.current) {
        skipNextUpdateRef.current = false
        lastContentRef.current = currentEditor.getText()
        return
      }

      const currentText = currentEditor.getText()
      const lastText = lastContentRef.current

      // Only log if content increased (new text added)
      if (currentText.length > lastText.length) {
        const insertedText = findInsertedText(lastText, currentText)
        if (insertedText.trim().length > 0) {
          // Log human typing event
          invoke('log_event', {
            eventType: 'human',
            text: insertedText,
            source: 'user'
          }).catch(console.error)
        }
      }

      lastContentRef.current = currentText
    }

    editor.on('update', handleUpdate)
    return () => editor.off('update', handleUpdate)
  }, [editor])

  // Helper function to find inserted text
  const findInsertedText = (oldText: string, newText: string): string => {
    // Simple diff algorithm - find the common prefix and suffix
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

  // Update citation confirm to skip next update
  const handleCitationConfirm = async (citation: string) => {
    if (!editor || !pendingPaste) return

    skipNextUpdateRef.current = true // Skip logging this insertion
    
    // ... rest of existing code
  }

  // ... rest of component
}
```

**Update**: `src/components/AssistantPanel.tsx` to skip logging AI insertions
```tsx
interface AssistantPanelProps {
  onInsertText?: (text: string) => void
  skipNextUpdate?: () => void
}

const AssistantPanel: React.FC<AssistantPanelProps> = ({ onInsertText, skipNextUpdate }) => {
  const insertIntoEditor = () => {
    if (lastResponse && onInsertText && skipNextUpdate) {
      skipNextUpdate() // Tell editor to skip logging this insertion
      onInsertText(lastResponse)
    }
  }
  // ... rest of component
}
```

**Update**: `src/App.tsx` to pass skip function
```tsx
const skipNextUpdateRef = useRef<() => void>()

const handleInsertText = (text: string) => {
  if (editorInstance) {
    // Mark the text as AI-generated
    editorInstance.chain()
      .focus()
      .setProvenance({ source: 'ai' })
      .insertContent(text)
      .unsetProvenance()
      .run()
  }
}

// In JSX
<EditorPane onReady={(editor) => {
  setEditorInstance(editor)
  skipNextUpdateRef.current = () => {
    // This function will be called to skip the next update
  }
}} />

<AssistantPanel 
  onInsertText={handleInsertText} 
  skipNextUpdate={skipNextUpdateRef.current}
/>
```

**Verify**: Human typing logs as 'human', AI insertions log as 'ai', pastes log as 'cited'

---

## Phase 4: Manifest Generation & Export (Days 7-8)

### Step 4.1: Build Manifest Calculation Logic (90 min)
**Create**: `src/utils/manifestGenerator.ts`
```typescript
export interface ManifestData {
  human_percentage: number
  ai_percentage: number
  cited_percentage: number
  total_characters: number
  events: ProvenanceEvent[]
  generated_at: string
  document_hash: string
}

export async function generateCompleteManifest(
  content: string,
  events: ProvenanceEvent[]
): Promise<ManifestData> {
  // Create a temporary DOM element to parse the HTML
  const tempDiv = document.createElement('div')
  tempDiv.innerHTML = content

  // Count total tokens
  const totalText = tempDiv.textContent || ''
  const totalTokens = countTokens(totalText)

  // Count AI tokens
  const aiElements = tempDiv.querySelectorAll('span[data-provenance="ai"]')
  const aiText = Array.from(aiElements).map(el => el.textContent || '').join(' ')
  const aiTokens = countTokens(aiText)

  // Count cited tokens
  const citedElements = tempDiv.querySelectorAll('span[data-provenance="cited"]')
  const citedText = Array.from(citedElements).map(el => el.textContent || '').join(' ')
  const citedTokens = countTokens(citedText)

  // Human tokens are the remainder
  const humanTokens = Math.max(0, totalTokens - aiTokens - citedTokens)

  // Calculate percentages
  const humanPercentage = totalTokens > 0 ? ((humanTokens / totalTokens) * 100).toFixed(1) + '%' : '0%'
  const aiPercentage = totalTokens > 0 ? ((aiTokens / totalTokens) * 100).toFixed(1) + '%' : '0%'
  const citedPercentage = totalTokens > 0 ? ((citedTokens / totalTokens) * 100).toFixed(1) + '%' : '0%'

  // Generate content hash
  const contentHash = generateContentHash(content)

  return {
    human_tokens: humanTokens,
    ai_tokens: aiTokens,
    cited_tokens: citedTokens,
    total_tokens: totalTokens,
    human_percentage: humanPercentage,
    ai_percentage: aiPercentage,
    cited_percentage: citedPercentage,
    generated_at: new Date().toISOString(),
    content_hash: contentHash
  }
}

function countTokens(text: string): number {
  if (!text.trim()) return 0
  // Simple word-based tokenization
  return text.trim().split(/\s+/).length
}

async function generateContentHash(content: string): Promise<string> {
  const encoder = new TextEncoder()
  const data = encoder.encode(content)
  const hashBuffer = await crypto.subtle.digest('SHA-256', data)
  const hashArray = Array.from(new Uint8Array(hashBuffer))
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('')
}
```

### Step 4.2: Create Manifest Display Modal (60 min)
**Create**: `src/components/ManifestModal.tsx`
```tsx
import React from 'react'
import { ManifestData } from '../utils/manifestGenerator'

interface ManifestModalProps {
  isOpen: boolean
  manifest: ManifestData | null
  onClose: () => void
  onExport?: () => void
}

const ManifestModal: React.FC<ManifestModalProps> = ({
  isOpen,
  manifest,
  onClose,
  onExport
}) => {
  if (!isOpen || !manifest) return null

  const copyToClipboard = () => {
    navigator.clipboard.writeText(JSON.stringify(manifest, null, 2))
  }

  return (
    <div className="modal-overlay">
      <div className="modal-content manifest-modal">
        <h3>Content Provenance Manifest</h3>
        
        <div className="manifest-summary">
          <div className="stat-grid">
            <div className="stat-item human">
              <div className="stat-value">{manifest.human_percentage}</div>
              <div className="stat-label">Human Written</div>
              <div className="stat-tokens">{manifest.human_tokens} tokens</div>
            </div>
            <div className="stat-item ai">
              <div className="stat-value">{manifest.ai_percentage}</div>
              <div className="stat-label">AI Generated</div>
              <div className="stat-tokens">{manifest.ai_tokens} tokens</div>
            </div>
            <div className="stat-item cited">
              <div className="stat-value">{manifest.cited_percentage}</div>
              <div className="stat-label">Externally Sourced</div>
              <div className="stat-tokens">{manifest.cited_tokens} tokens</div>
            </div>
          </div>
        </div>

        <div className="manifest-details">
          <h4>Full Manifest</h4>
          <pre className="manifest-json">
            {JSON.stringify(manifest, null, 2)}
          </pre>
        </div>

        <div className="modal-actions">
          <button onClick={copyToClipboard} className="copy-btn">
            Copy JSON
          </button>
          {onExport && (
            <button onClick={onExport} className="export-btn">
              Export Signed Document
            </button>
          )}
          <button onClick={onClose} className="close-btn">
            Close
          </button>
        </div>
      </div>
    </div>
  )
}

export default ManifestModal
```

**Update**: `src/App.css` with manifest modal styles
```css
.manifest-modal {
  width: 600px;
  max-height: 80vh;
  overflow-y: auto;
}

.manifest-summary {
  margin: 20px 0;
}

.stat-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}

.stat-item {
  text-align: center;
  padding: 16px;
  border-radius: 8px;
  border: 2px solid;
}

.stat-item.human {
  border-color: #28a745;
  background: #f8fff9;
}

.stat-item.ai {
  border-color: #fd7e14;
  background: #fff8f0;
}

.stat-item.cited {
  border-color: #007bff;
  background: #f0f8ff;
}

.stat-value {
  font-size: 24px;
  font-weight: bold;
  margin-bottom: 4px;
}

.stat-label {
  font-size: 14px;
  color: #666;
  margin-bottom: 2px;
}

.stat-tokens {
  font-size: 12px;
  color: #888;
}

.manifest-details {
  margin: 20px 0;
}

.manifest-json {
  background: #f8f9fa;
  padding: 16px;
  border-radius: 6px;
  font-size: 12px;
  max-height: 300px;
  overflow-y: auto;
}

.copy-btn {
  background: #6c757d;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
}

.export-btn {
  background: #28a745;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
}

.close-btn {
  background: #007bff;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
}
```

### Step 4.3: Integrate Manifest Generation (45 min)
**Update**: `src/App.tsx` with manifest functionality
```tsx
import { generateCompleteManifest, ManifestData } from './utils/manifestGenerator'
import ManifestModal from './components/ManifestModal'

function App() {
  const [editorInstance, setEditorInstance] = useState(null)
  const [manifestData, setManifestData] = useState<ManifestData | null>(null)
  const [showManifest, setShowManifest] = useState(false)

  const handlePublish = async () => {
    if (!editorInstance) return
    
    try {
      const html = editorInstance.getHTML()
      const manifest = await generateCompleteManifest(html, events)
      setManifestData(manifest)
      setShowManifest(true)
    } catch (error) {
      console.error('Error generating manifest:', error)
    }
  }

  const handleExport = () => {
    // This will be implemented in Phase 5
    console.log('Export functionality coming in Phase 5')
  }

  return (
    <div className="app-container">
      {/* ... existing JSX ... */}
      <div className="toolbar">
        <button onClick={handlePublish} className="publish-btn">
          Generate Manifest
        </button>
      </div>

      <ManifestModal
        isOpen={showManifest}
        manifest={manifestData}
        onClose={() => setShowManifest(false)}
        onExport={handleExport}
      />
    </div>
  )
}
```

**Verify**: Publish button generates accurate manifest with percentages

### Step 4.4: Add Visual Provenance Indicators (60 min)
**Update**: `src/App.css` with provenance styling
```css
/* Provenance visual indicators */
span[data-provenance="ai"] {
  background: linear-gradient(90deg, rgba(253, 126, 20, 0.1) 0%, rgba(253, 126, 20, 0.05) 100%);
  border-left: 3px solid #fd7e14;
  padding-left: 4px;
  margin-left: 2px;
}

span[data-provenance="cited"] {
  background: linear-gradient(90deg, rgba(0, 123, 255, 0.1) 0%, rgba(0, 123, 255, 0.05) 100%);
  border-left: 3px solid #007bff;
  padding-left: 4px;
  margin-left: 2px;
}

/* Hover effects for provenance spans */
span[data-provenance="ai"]:hover::after {
  content: "AI Generated";
  position: absolute;
  background: #fd7e14;
  color: white;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 11px;
  margin-left: 8px;
  z-index: 100;
}

span[data-provenance="cited"]:hover::after {
  content: "External Source";
  position: absolute;
  background: #007bff;
  color: white;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 11px;
  margin-left: 8px;
  z-index: 100;
}
```

**Create**: `src/components/ProvenanceLegend.tsx`
```tsx
import React from 'react'

const ProvenanceLegend: React.FC = () => {
  return (
    <div className="provenance-legend">
      <h4>Content Sources</h4>
      <div className="legend-items">
        <div className="legend-item">
          <span className="legend-indicator human"></span>
          <span>Human Written</span>
        </div>
        <div className="legend-item">
          <span className="legend-indicator ai"></span>
          <span>AI Generated</span>
        </div>
        <div className="legend-item">
          <span className="legend-indicator cited"></span>
          <span>External Source</span>
        </div>
      </div>
    </div>
  )
}

export default ProvenanceLegend
```

**Update**: `src/App.css` with legend styles
```css
.provenance-legend {
  margin-top: 20px;
  padding: 16px;
  background: #f8f9fa;
  border-radius: 6px;
}

.legend-items {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
}

.legend-indicator {
  width: 16px;
  height: 16px;
  border-radius: 2px;
}

.legend-indicator.human {
  background: #28a745;
}

.legend-indicator.ai {
  background: #fd7e14;
}

.legend-indicator.cited {
  background: #007bff;
}
```

**Update**: `src/App.tsx` to include legend
```tsx
import ProvenanceLegend from './components/ProvenanceLegend'

// In assistant-pane
<div className="assistant-pane">
  <h2>AI Assistant</h2>
  <AssistantPanel 
    onInsertText={handleInsertText} 
    skipNextUpdate={skipNextUpdateRef.current}
  />
  <ProvenanceLegend />
</div>
```

**Verify**: Different content types have visual indicators, legend explains them

---

## Phase 5: Cryptographic Signing (Days 9-10)

### Step 5.1: Add Crypto Dependencies (30 min)
**Update**: `src-tauri/Cargo.toml`
```toml
[dependencies]
# ... existing dependencies ...
ed25519-dalek = "2.0"
rand = "0.8"
base64 = "0.21"
serde_json = "1.0"
```

### Step 5.2: Implement Key Generation (75 min)
**Update**: `src-tauri/src/main.rs` with crypto functions
```rust
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use base64::{Engine as _, engine::general_purpose};
use std::fs;

#[derive(serde::Serialize, serde::Deserialize)]
struct KeyPair {
    public_key: String,
    private_key: String,
}

#[tauri::command]
fn generate_keypair() -> Result<KeyPair, String> {
    let mut csprng = OsRng {};
    let keypair: Keypair = Keypair::generate(&mut csprng);
    
    let public_key = general_purpose::STANDARD.encode(keypair.public.as_bytes());
    let private_key = general_purpose::STANDARD.encode(keypair.secret.as_bytes());
    
    Ok(KeyPair {
        public_key,
        private_key,
    })
}

#[tauri::command]
fn save_keypair(keypair: KeyPair) -> Result<(), String> {
    let keys_dir = "keys";
    fs::create_dir_all(keys_dir).map_err(|e| e.to_string())?;
    
    fs::write(
        format!("{}/public_key.txt", keys_dir),
        &keypair.public_key
    ).map_err(|e| e.to_string())?;
    
    fs::write(
        format!("{}/private_key.txt", keys_dir),
        &keypair.private_key
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
fn load_keypair() -> Result<KeyPair, String> {
    let public_key = fs::read_to_string("keys/public_key.txt")
        .map_err(|_| "Public key not found")?;
    let private_key = fs::read_to_string("keys/private_key.txt")
        .map_err(|_| "Private key not found")?;
    
    Ok(KeyPair {
        public_key: public_key.trim().to_string(),
        private_key: private_key.trim().to_string(),
    })
}
```

### Step 5.3: Implement Document Signing (90 min)
**Update**: `src-tauri/src/main.rs` with signing functions
```rust
#[derive(serde::Serialize, serde::Deserialize)]
struct SignedManifest {
    manifest: serde_json::Value,
    signature: String,
    public_key: String,
    timestamp: String,
}

#[tauri::command]
fn sign_document(manifest_json: String) -> Result<SignedManifest, String> {
    // Load private key
    let keypair = load_keypair()?;
    let private_key_bytes = general_purpose::STANDARD
        .decode(&keypair.private_key)
        .map_err(|e| format!("Invalid private key: {}", e))?;
    
    let secret_key = SecretKey::from_bytes(&private_key_bytes)
        .map_err(|e| format!("Invalid secret key: {}", e))?;
    
    let public_key_bytes = general_purpose::STANDARD
        .decode(&keypair.public_key)
        .map_err(|e| format!("Invalid public key: {}", e))?;
    
    let public_key = PublicKey::from_bytes(&public_key_bytes)
        .map_err(|e| format!("Invalid public key: {}", e))?;
    
    let keypair = Keypair {
        secret: secret_key,
        public: public_key,
    };
    
    // Parse and normalize the manifest
    let manifest: serde_json::Value = serde_json::from_str(&manifest_json)
        .map_err(|e| format!("Invalid manifest JSON: {}", e))?;
    
    // Create canonical JSON for signing
    let canonical_manifest = serde_json::to_string(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
    
    // Sign the manifest
    let signature = keypair.sign(canonical_manifest.as_bytes());
    let signature_b64 = general_purpose::STANDARD.encode(signature.to_bytes());
    
    Ok(SignedManifest {
        manifest,
        signature: signature_b64,
        public_key: keypair.public_key,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

#[tauri::command]
fn export_signed_document(content_html: String, signed_manifest: SignedManifest) -> Result<String, String> {
    let export_html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Signed Document</title>
    <style>
        body {{ font-family: system-ui, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        .provenance-header {{ background: #f8f9fa; padding: 16px; border-radius: 8px; margin-bottom: 20px; }}
        .signature-info {{ font-family: monospace; font-size: 12px; color: #666; }}
        span[data-provenance="ai"] {{ background: rgba(253, 126, 20, 0.1); border-left: 3px solid #fd7e14; padding-left: 4px; }}
        span[data-provenance="cited"] {{ background: rgba(0, 123, 255, 0.1); border-left: 3px solid #007bff; padding-left: 4px; }}
    </style>
</head>
<body>
    <div class="provenance-header">
        <h3>🔒 Cryptographically Signed Content</h3>
        <div class="signature-info">
            <p><strong>Public Key:</strong> {}</p>
            <p><strong>Signature:</strong> {}</p>
            <p><strong>Signed:</strong> {}</p>
        </div>
    </div>
    
    <div class="content">
        {}
    </div>
    
    <script type="application/json" id="sonnun-manifest">
        {}
    </script>
</body>
</html>"#,
        signed_manifest.public_key,
        signed_manifest.signature,
        signed_manifest.timestamp,
        content_html,
        serde_json::to_string_pretty(&signed_manifest).unwrap()
    );
    
    // Save to file
    let filename = format!("signed_document_{}.html", 
        chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    fs::write(&filename, &export_html).map_err(|e| e.to_string())?;
    
    Ok(filename)
}

// Update main function to include new commands
fn main() {
    tauri::Builder::default()
        .plugin(SqlBuilder::default()
            .add_migrations("sqlite:sonnun.db", create_migrations())
            .build())
        .invoke_handler(tauri::generate_handler![
            test_command,
            openai_complete,
            log_event,
            generate_keypair,
            save_keypair,
            load_keypair,
            sign_document,
            export_signed_document
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Step 5.4: Add Key Management UI (60 min)
**Create**: `src/components/KeyManager.tsx`
```tsx
import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'

interface KeyPair {
  public_key: string
  private_key: string
}

const KeyManager: React.FC = () => {
  const [hasKeys, setHasKeys] = useState(false)
  const [publicKey, setPublicKey] = useState('')
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    checkForKeys()
  }, [])

  const checkForKeys = async () => {
    try {
      const keypair = await invoke<KeyPair>('load_keypair')
      setPublicKey(keypair.public_key)
      setHasKeys(true)
    } catch (error) {
      setHasKeys(false)
    }
  }

  const generateKeys = async () => {
    setLoading(true)
    try {
      const keypair = await invoke<KeyPair>('generate_keypair')
      await invoke('save_keypair', { keypair })
      setPublicKey(keypair.public_key)
      setHasKeys(true)
    } catch (error) {
      console.error('Key generation failed:', error)
    } finally {
      setLoading(false)
    }
  }

  const copyPublicKey = () => {
    navigator.clipboard.writeText(`prov-pk: ed25519:${publicKey}`)
  }

  return (
    <div className="key-manager">
      <h4>Cryptographic Keys</h4>
      {hasKeys ? (
        <div className="key-info">
          <p className="key-status">✅ Keys generated and saved</p>
          <div className="public-key-display">
            <label>Public Key (for your bio):</label>
            <input 
              readOnly 
              value={`prov-pk: ed25519:${publicKey.substring(0, 20)}...`}
              className="public-key-input"
            />
            <button onClick={copyPublicKey} className="copy-key-btn">
              Copy
            </button>
          </div>
        </div>
      ) : (
        <div className="no-keys">
          <p>No cryptographic keys found.</p>
          <button 
            onClick={generateKeys} 
            disabled={loading}
            className="generate-keys-btn"
          >
            {loading ? 'Generating...' : 'Generate Keys'}
          </button>
        </div>
      )}
    </div>
  )
}

export default KeyManager
```

**Update**: `src/App.css` with key manager styles
```css
.key-manager {
  margin-top: 20px;
  padding: 16px;
  background: #f0f8ff;
  border-radius: 6px;
  border: 1px solid #007bff;
}

.key-status {
  color: #28a745;
  font-weight: 500;
  margin: 8px 0;
}

.public-key-display {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.public-key-input {
  font-family: monospace;
  font-size: 11px;
  padding: 8px;
  border: 1px solid #ccc;
  border-radius: 4px;
  background: white;
}

.copy-key-btn, .generate-keys-btn {
  background: #007bff;
  color: white;
  border: none;
  padding: 8px 12px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}

.no-keys {
  text-align: center;
  color: #666;
}
```

### Step 5.5: Integrate Signing into Export (45 min)
**Update**: `src/App.tsx` with signing functionality
```tsx
import KeyManager from './components/KeyManager'

const handleExport = async () => {
  if (!editorInstance || !manifestData) return
  
  try {
    setLoading(true)
    
    // Sign the manifest
    const signedManifest = await invoke('sign_document', {
      manifestJson: JSON.stringify(manifestData)
    })
    
    // Export the signed document
    const html = editorInstance.getHTML()
    const filename = await invoke('export_signed_document', {
      contentHtml: html,
      signedManifest
    })
    
    alert(`Document exported as: ${filename}`)
    
  } catch (error) {
    console.error('Export failed:', error)
    alert('Export failed: ' + error)
  } finally {
    setLoading(false)
  }
}

// In assistant-pane JSX
<div className="assistant-pane">
  <h2>AI Assistant</h2>
  <AssistantPanel 
    onInsertText={handleInsertText} 
    skipNextUpdate={skipNextUpdateRef.current}
  />
  <ProvenanceLegend />
  <KeyManager />
</div>
```

**Verify**: Keys generate, manifests sign, documents export with embedded signatures

---

## Phase 6: Verification & Distribution (Days 11-12)

### Step 6.1: Build Standalone Verifier (90 min)
**Create**: `src-tauri/src/bin/verify.rs`
```rust
use clap::{Arg, Command};
use std::fs;
use std::io::{self, Read};
use serde_json::Value;
use ed25519_dalek::{PublicKey, Signature, Verifier};
use base64::{Engine as _, engine::general_purpose};

fn main() {
    let matches = Command::new("sonnun-verify")
        .version("1.0")
        .about("Verify Sonnun signed documents")
        .arg(Arg::new("file")
            .help("HTML file to verify")
            .required(true)
            .index(1))
        .arg(Arg::new("public-key")
            .short('k')
            .long("key")
            .value_name("KEY")
            .help("Public key to verify against (base64)"))
        .get_matches();

    let filename = matches.get_one::<String>("file").unwrap();
    let provided_key = matches.get_one::<String>("public-key");

    match verify_document(filename, provided_key) {
        Ok(result) => {
            if result.valid {
                println!("✅ VALID signature");
                println!("Public key: {}", result.public_key);
                println!("Manifest: {}", serde_json::to_string_pretty(&result.manifest).unwrap());
            } else {
                println!("❌ INVALID signature");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

struct VerificationResult {
    valid: bool,
    public_key: String,
    manifest: Value,
}

fn verify_document(filename: &str, provided_key: Option<&String>) -> Result<VerificationResult, String> {
    // Read the HTML file
    let content = fs::read_to_string(filename)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Extract the JSON manifest from the script tag
    let manifest_start = content.find(r#"<script type="application/json" id="sonnun-manifest">"#)
        .ok_or("No Sonnun manifest found in document")?;
    let manifest_content_start = content[manifest_start..].find(">")
        .ok_or("Malformed manifest script tag")?;
    let manifest_content_end = content[manifest_start..].find("</script>")
        .ok_or("Manifest script tag not closed")?;

    let manifest_json = &content[manifest_start + manifest_content_start + 1..manifest_start + manifest_content_end];
    let signed_manifest: Value = serde_json::from_str(manifest_json.trim())
        .map_err(|e| format!("Invalid manifest JSON: {}", e))?;

    // Extract components
    let manifest = signed_manifest["manifest"].clone();
    let signature_b64 = signed_manifest["signature"].as_str()
        .ok_or("No signature in manifest")?;
    let public_key_b64 = signed_manifest["public_key"].as_str()
        .ok_or("No public key in manifest")?;

    // If a public key was provided, verify it matches
    if let Some(key) = provided_key {
        if key != public_key_b64 {
            return Err("Provided public key does not match document key".to_string());
        }
    }

    // Decode public key and signature
    let public_key_bytes = general_purpose::STANDARD.decode(public_key_b64)
        .map_err(|e| format!("Invalid public key encoding: {}", e))?;
    let signature_bytes = general_purpose::STANDARD.decode(signature_b64)
        .map_err(|e| format!("Invalid signature encoding: {}", e))?;

    let public_key = PublicKey::from_bytes(&public_key_bytes)
        .map_err(|e| format!("Invalid public key: {}", e))?;
    let signature = Signature::from_bytes(&signature_bytes)
        .map_err(|e| format!("Invalid signature: {}", e))?;

    // Verify signature
    let canonical_manifest = serde_json::to_string(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;

    let valid = public_key.verify(canonical_manifest.as_bytes(), &signature).is_ok();

    Ok(VerificationResult {
        valid,
        public_key: public_key_b64.to_string(),
        manifest,
    })
}
```

**Update**: `src-tauri/Cargo.toml` to build verifier
```toml
[[bin]]
name = "sonnun-verify"
path = "src/bin/verify.rs"

[dependencies]
# ... existing dependencies ...
clap = "4.0"
```

**Test verifier**:
```bash
cd src-tauri
cargo build --bin sonnun-verify
./target/debug/sonnun-verify ../signed_document_20241201_123456.html
```

### Step 6.2: Create Badge Generation Service (60 min)
**Create**: `badge-service/` directory with simple Node.js service
**Create**: `badge-service/package.json`
```json
{
  "name": "sonnun-badge-service",
  "version": "1.0.0",
  "scripts": {
    "start": "node server.js"
  },
  "dependencies": {
    "express": "^4.18.0",
    "node-fetch": "^3.3.0"
  }
}
```

**Create**: `badge-service/server.js`
```javascript
const express = require('express')
const fetch = require('node-fetch')
const app = express()

app.get('/badge/:hash.svg', async (req, res) => {
  const { hash } = req.params
  
  try {
    // In a real implementation, you'd verify the document
    // For now, we'll generate a basic SVG badge
    const svg = generateBadgeSVG(hash)
    
    res.setHeader('Content-Type', 'image/svg+xml')
    res.setHeader('Cache-Control', 'public, max-age=3600')
    res.send(svg)
  } catch (error) {
    res.status(500).send('Badge generation failed')
  }
})

app.get('/verify', async (req, res) => {
  const { url, hash } = req.query
  
  try {
    if (url) {
      // Fetch and verify document from URL
      const response = await fetch(url)
      const html = await response.text()
      const result = verifyDocumentHtml(html)
      res.json(result)
    } else if (hash) {
      // Look up verification result by hash
      res.json({ error: 'Hash lookup not implemented' })
    } else {
      res.status(400).json({ error: 'url or hash parameter required' })
    }
  } catch (error) {
    res.status(500).json({ error: error.message })
  }
})

function generateBadgeSVG(hash) {
  return `<svg xmlns="http://www.w3.org/2000/svg" width="150" height="20">
    <rect width="150" height="20" fill="#007bff"/>
    <text x="75" y="15" font-family="Arial" font-size="12" fill="white" text-anchor="middle">
      Sonnun Verified
    </text>
    <metadata>
      <sonnun:hash>${hash}</sonnun:hash>
    </metadata>
  </svg>`
}

function verifyDocumentHtml(html) {
  // Simplified verification - in production this would use the full crypto verification
  const hasManifest = html.includes('sonnun-manifest')
  const hasSignature = html.includes('"signature"')
  
  return {
    valid: hasManifest && hasSignature,
    result: hasManifest && hasSignature ? 'VALID' : 'INVALID',
    timestamp: new Date().toISOString()
  }
}

const PORT = process.env.PORT || 3001
app.listen(PORT, () => {
  console.log(`Badge service running on port ${PORT}`)
})
```

### Step 6.3: Add Public Key Distribution (45 min)
**Create**: `public-key-service/` directory
**Create**: `public-key-service/.well-known/prov.json`
```json
{
  "pubkey": "REPLACE_WITH_ACTUAL_PUBLIC_KEY",
  "algorithm": "ed25519",
  "version": "1.0",
  "created": "2024-12-01T00:00:00Z"
}
```

**Update**: Export functionality to include public key instructions
**Update**: `src/components/ManifestModal.tsx`
```tsx
const ManifestModal: React.FC<ManifestModalProps> = ({ /* ... */ }) => {
  const copyBioText = () => {
    // This would use the actual public key from the app
    navigator.clipboard.writeText("prov-pk: ed25519:MCowBQYDK2VwAyEA...")
  }

  return (
    <div className="modal-overlay">
      <div className="modal-content manifest-modal">
        {/* ... existing content ... */}
        
        <div className="distribution-section">
          <h4>📢 Public Key Distribution</h4>
          <p>To enable verification, add this to your profile bio:</p>
          <div className="bio-text">
            <code>prov-pk: ed25519:MCowBQYDK2VwAyEA...</code>
            <button onClick={copyBioText} className="copy-btn">Copy</button>
          </div>
          <p><small>Supported platforms: GitHub, Twitter, HN, personal websites</small></p>
        </div>

        {/* ... rest of modal ... */}
      </div>
    </div>
  )
}
```

### Step 6.4: Final Testing & Integration (90 min)
**Create comprehensive test document**:
1. Type some human text
2. Insert AI-generated content
3. Paste external content with citation
4. Generate manifest
5. Export signed document
6. Verify with standalone tool
7. Test badge generation

**Create**: `test-workflow.md` documentation
```markdown
# Sonnun Test Workflow

## 1. Content Creation Test
- [ ] Human typing logs correctly
- [ ] AI insertion marks content as AI
- [ ] Paste requires citation
- [ ] Visual indicators show correctly

## 2. Manifest Generation Test
- [ ] Percentages calculate correctly
- [ ] All content types counted
- [ ] JSON structure is valid

## 3. Signing & Export Test
- [ ] Keys generate successfully
- [ ] Document signs without errors
- [ ] Exported HTML includes manifest
- [ ] File saves correctly

## 4. Verification Test
- [ ] Standalone verifier works
- [ ] Valid signatures pass
- [ ] Invalid signatures fail
- [ ] Public key matching works

## 5. Integration Test
- [ ] Full workflow end-to-end
- [ ] Badge service responds
- [ ] Public key distribution ready
```

**Run full test suite**:
```bash
# Build everything
npm run tauri build
cd src-tauri && cargo build --bin sonnun-verify
cd ../badge-service && npm install && npm start

# Test the full workflow
# 1. Create content with all types
# 2. Generate and verify manifest
# 3. Export signed document
# 4. Verify with CLI tool
# 5. Test badge generation
```

**Verify**: Complete end-to-end workflow works from content creation to verification

---

## Deployment Checklist

### Phase 7: Production Deployment (Day 13)

1. **Desktop App Packaging**
   - [ ] Build for macOS, Windows, Linux
   - [ ] Test on each platform
   - [ ] Create installers/packages

2. **Verifier Distribution**
   - [ ] Package CLI tool for distribution
   - [ ] Create installation scripts
   - [ ] Document usage

3. **Badge Service Deployment**
   - [ ] Deploy to cloud service (Vercel/Netlify)
   - [ ] Set up domain
   - [ ] Test public endpoints

4. **Documentation**
   - [ ] User guide
   - [ ] Developer docs
   - [ ] Integration examples

**Estimated Total Time**: 13-16 days for complete implementation

This plan provides a comprehensive, step-by-step implementation guide with specific files, commands, and verification steps for each phase of the Sonnun project.