# Sonnun: Provably Honest Content Creation

> A desktop markdown editor that cryptographically proves content provenance - distinguishing human
> writing from AI assistance and external sources.

## Vision

Combat "AI slop" by making honest attribution easier than deception. Writers get a tool that
transparently tracks content origins; readers get cryptographic proof of authorship.

## Trust Chain Overview

```
author laptop ──signs──► markdown.html + c2pa.json
                 ▲                         │
                 │                         ▼
          private key (ed25519)   public key in profile bio
```

**Verification flow**: Reader pulls pubkey from author's bio → verifies manifest hash → gets
transparency report

## Core Architecture

### Desktop App (Tauri + React)

- **Left pane**: Tiptap markdown editor with real-time provenance tracking
- **Right pane**: ChatGPT assistant for content generation
- **Event logger**: SQLite database recording every text insertion with source attribution
- **Export**: Generates signed HTML + C2PA manifest

### Key Features

1. **Citation enforcement**: Paste operations require source attribution
2. **AI transparency**: All AI-generated content visibly marked and logged
3. **Cryptographic signing**: Ed25519 signatures prove document integrity
4. **Manifest generation**: Shows exact percentages of human/AI/cited content

## Tech Stack

- **Frontend**: React, TypeScript, Tiptap editor
- **Backend**: Rust, Tauri framework
- **Database**: SQLite with append-only event log
- **AI**: OpenAI API integration
- **Crypto**: ed25519 signatures, C2PA standard
- **Distribution**: Public key in author bio + optional webfinger

## Implementation Phases

| Phase            | Deliverable                     | Components                                      |
| ---------------- | ------------------------------- | ----------------------------------------------- |
| **MVP**          | Desktop editor with AI chat     | Tauri app, Tiptap, OpenAI proxy, SQLite logging |
| **Signing**      | Export with cryptographic proof | ed25519 keygen, C2PA manifest, HTML export      |
| **Verification** | Standalone verifier tool        | CLI tool for manifest validation                |
| **Distribution** | Public verification badges      | Web service for badge generation                |
| **Ecosystem**    | Editor plugins                  | Obsidian, VS Code, static site integrations     |

## Content Attribution Model

All text is categorized and tracked:

```typescript
if (event.agent === 'ai') ai_tokens += span_length
else if (event.agent === 'cited') cited_tokens += span_length
else human_tokens += span_length
```

Manifest includes both raw counts and percentages to prevent gaming.

## Key Distribution

1. **Profile embedding**: `prov-pk: ed25519:MCowBQYDK2VwAyEAh2T…X==` in bio
2. **Webfinger fallback**: `/.well-known/prov.json` on personal domain
3. **Badge verification**: SVG badges include embedded pubkey hash

Verification is a single curl command:

```bash
curl -s https://prove.dev/verify?url=https://blog.com/post.html | jq .result
```

## Detailed Implementation Guide

### 1. Project Setup

**Initialize Tauri project:**

```bash
npm create tauri-app@latest sonnun
cd sonnun
npm install
```

**Add required dependencies:**

```bash
# Frontend
npm install @tiptap/react @tiptap/starter-kit @tiptap/extension-link @tauri-apps/api

# Backend (add to Cargo.toml)
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

### 2. Core Components

#### Frontend Architecture

- **App.tsx**: Main layout with editor and assistant panels
- **EditorPane.tsx**: Tiptap editor with provenance tracking
- **AssistantPanel.tsx**: ChatGPT integration sidebar
- **ProvenanceModal.tsx**: Citation requirement for pasted content

#### Backend Commands

- `openai_complete(prompt)`: Proxy to OpenAI API with logging
- `log_event(type, text, source)`: Record provenance events
- `export_document(content)`: Generate signed manifest

### 3. Database Schema

```sql
CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    event_type TEXT NOT NULL, -- 'human', 'ai', 'cited'
    text_hash TEXT NOT NULL,
    source TEXT,             -- 'user', model name, or citation URL
    span_length INTEGER
);
```

### 4. Content Provenance Flow

1. **Human typing**: Tracked via Tiptap update events, logged as `human` type
2. **AI generation**: Triggered from assistant panel, marked with `data-provenance="ai"`
3. **External paste**: Intercepted, requires citation modal, marked as `cited`
4. **Export**: Generates manifest showing exact percentages

### 5. Development Phases

**Phase 1: Basic Editor** (Days 1-2)

- Tauri app skeleton
- Tiptap editor integration
- Simple layout

**Phase 2: AI Integration** (Days 3-4)

- OpenAI API proxy
- Assistant sidebar
- Event logging system

**Phase 3: Provenance Tracking** (Days 5-6)

- Citation enforcement
- Manifest generation
- Visual provenance indicators

**Phase 4: Signing & Export** (Days 7-8)

- Ed25519 key generation
- C2PA manifest creation
- HTML export with embedded proof

### 6. Testing Strategy

- Unit tests for provenance calculations
- Integration tests for AI API calls
- Manual testing of citation flow
- Cryptographic signature verification

### 7. Deployment

- Tauri native builds for macOS/Windows/Linux
- Standalone verifier CLI tool
- Web-based badge generation service

## Sample Implementation

### Frontend App Structure

**App.tsx:**

```tsx
import React, { useState, useRef, useEffect } from 'react'
import { Editor } from '@tiptap/core'
import { invoke } from '@tauri-apps/api'

function App() {
  const [editorInstance, setEditorInstance] = useState<Editor | null>(null)
  const [manifestData, setManifestData] = useState<any>(null)
  const skipLogRef = useRef(false)

  useEffect(() => {
    if (!editorInstance) return
    let lastText = editorInstance.getText()

    editorInstance.on('update', ({ editor }) => {
      if (skipLogRef.current) {
        skipLogRef.current = false
        lastText = editor.getText()
        return
      }

      const newText = editor.getText()
      if (newText.length <= lastText.length) {
        lastText = newText
        return
      }

      // Compute diff and log human insertions
      const insertedText = computeTextDiff(lastText, newText)
      if (insertedText.trim()) {
        invoke('log_event', { type: 'human', text: insertedText, source: 'user' })
      }
      lastText = newText
    })
  }, [editorInstance])

  const onPublish = () => {
    if (!editorInstance) return
    const html = editorInstance.getHTML()
    const manifest = generateProvenanceManifest(html)
    setManifestData(manifest)
  }

  return (
    <div className="app-container">
      <EditorPane onReady={setEditorInstance} skipLogRef={skipLogRef} />
      <AssistantPanel editor={editorInstance} skipLogRef={skipLogRef} />
      <button onClick={onPublish}>Publish</button>
      {manifestData && <ManifestModal data={manifestData} onClose={() => setManifestData(null)} />}
    </div>
  )
}
```

### Tiptap Editor with Provenance

**EditorPane.tsx:**

```tsx
import React, { useState, useEffect } from 'react'
import { useEditor, EditorContent } from '@tiptap/react'
import StarterKit from '@tiptap/starter-kit'
import Link from '@tiptap/extension-link'
import { Mark } from '@tiptap/core'

const ProvenanceMark = Mark.create({
  name: 'provenance',
  addAttributes() {
    return { source: { default: null } }
  },
  parseHTML() {
    return [{ tag: 'span[data-provenance]' }]
  },
  renderHTML({ mark, HTMLAttributes }) {
    const { source } = mark.attrs
    return ['span', { ...HTMLAttributes, 'data-provenance': source }, 0]
  },
  inclusive() {
    return false
  },
})

const EditorPane = ({ onReady, skipLogRef }) => {
  const [pendingPaste, setPendingPaste] = useState(null)
  const [citation, setCitation] = useState('')

  const editor = useEditor({
    extensions: [StarterKit, Link, ProvenanceMark],
    editorProps: {
      handlePaste(view, event) {
        event.preventDefault()
        const text = event.clipboardData?.getData('text/plain') || ''
        if (text) setPendingPaste(text)
        return true
      },
    },
  })

  useEffect(() => {
    if (editor && onReady) onReady(editor)
  }, [editor])

  const confirmPasteInsert = () => {
    if (!editor || !pendingPaste || !citation.trim()) return
    const isURL = citation.startsWith('http')

    skipLogRef.current = true
    editor.chain().focus().setMark('provenance', { source: 'cited' }).insertText(pendingPaste).run()

    if (isURL) {
      editor.commands.setTextSelection({
        from: editor.state.selection.from - pendingPaste.length,
        to: editor.state.selection.from,
      })
      editor.commands.setLink({ href: citation })
    } else {
      editor.commands.insertContent(` (source: ${citation})`)
    }

    editor.commands.unsetMark('provenance')
    invoke('log_event', { type: 'cited', text: pendingPaste, source: citation })
    setPendingPaste(null)
    setCitation('')
  }

  return (
    <div className="editor-pane">
      <EditorContent editor={editor} />
      {pendingPaste && (
        <div className="citation-modal">
          <p>Provide source for pasted content:</p>
          <input
            value={citation}
            onChange={(e) => setCitation(e.target.value)}
            placeholder="URL or source description"
          />
          <button onClick={confirmPasteInsert}>Add Citation</button>
          <button onClick={() => setPendingPaste(null)}>Cancel</button>
        </div>
      )}
    </div>
  )
}
```

### AI Assistant Integration

**AssistantPanel.tsx:**

```tsx
import React, { useState } from 'react'
import { invoke } from '@tauri-apps/api'

const AssistantPanel = ({ editor, skipLogRef }) => {
  const [prompt, setPrompt] = useState('')
  const [loading, setLoading] = useState(false)

  const handleAskAI = async () => {
    if (!editor || !prompt.trim() || loading) return

    setLoading(true)
    try {
      const result = await invoke('openai_complete', { prompt })
      if (result) {
        skipLogRef.current = true
        editor
          .chain()
          .focus()
          .setMark('provenance', { source: 'ai' })
          .insertText(result)
          .unsetMark('provenance')
          .run()
      }
    } catch (error) {
      console.error('AI request failed:', error)
    } finally {
      setLoading(false)
      setPrompt('')
    }
  }

  return (
    <div className="assistant-panel">
      <h3>AI Assistant</h3>
      <textarea
        value={prompt}
        onChange={(e) => setPrompt(e.target.value)}
        placeholder="Ask for help or content..."
        disabled={loading}
      />
      <button onClick={handleAskAI} disabled={!prompt.trim() || loading}>
        {loading ? 'Generating...' : 'Ask AI'}
      </button>
    </div>
  )
}
```

### Backend Implementation

**src-tauri/src/main.rs:**

```rust
use tauri::Manager;
use tauri_plugin_sql::{Builder as SqlBuilder, Migration, MigrationKind};
use reqwest::Client;
use serde_json::Value;
use sha2::{Sha256, Digest};

#[tauri::command]
async fn openai_complete(prompt: String) -> Result<String, String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OpenAI API key not found")?;

    let client = Client::new();
    let payload = serde_json::json!({
        "model": "gpt-3.5-turbo",
        "messages": [{ "role": "user", "content": prompt }]
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let json: Value = response.json().await
        .map_err(|e| format!("JSON parse error: {}", e))?;

    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    if !content.is_empty() {
        let model = json["model"].as_str().unwrap_or("gpt-3.5-turbo");
        log_event_internal("ai", &content, model).ok();
    }

    Ok(content)
}

#[tauri::command]
fn log_event(event_type: &str, text: &str, source: &str) -> Result<(), String> {
    log_event_internal(event_type, text, source)
}

fn log_event_internal(event_type: &str, text: &str, source: &str) -> Result<(), String> {
    let hash = format!("{:x}", Sha256::digest(text.as_bytes()));
    let timestamp = chrono::Utc::now().to_rfc3339();

    let conn = rusqlite::Connection::open("sonnun.db")
        .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO events (timestamp, event_type, text_hash, source, span_length) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![timestamp, event_type, hash, source, text.len()]
    ).map_err(|e| e.to_string())?;

    Ok(())
}

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
                span_length INTEGER
            );",
        kind: MigrationKind::Up,
    }]
}

fn main() {
    tauri::Builder::default()
        .plugin(SqlBuilder::default().add_migrations("sqlite:sonnun.db", create_migrations()).build())
        .invoke_handler(tauri::generate_handler![openai_complete, log_event])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

This implementation provides a solid foundation for the provenance-aware markdown editor with clear
separation of concerns and modular architecture.
