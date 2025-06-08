# CLAUDE.md. Julep AI
*Last updated 2025-06-08*

> **purpose** – This file is the onboarding manual for every AI assistant (Claude, Cursor, GPT, etc.) and every human who edits this repository.
> It encodes our coding standards, guard-rails, and workflow tricks so the *human 30 %* (architecture, tests, domain judgment) stays in human hands.[^1]

---

## 0. Project overview

**Sonnun** is a provenance-aware markdown editor built with Tauri + React that cryptographically proves content authorship. The app distinguishes between human writing, AI assistance, and external sources, providing transparency through:

- **Real-time provenance tracking**: Every text insertion is logged with source attribution (human/AI/cited)
- **Citation enforcement**: Paste operations require source attribution via modal dialogs
- **Visual indicators**: Different content types are visually marked in the editor
- **Cryptographic signing**: Documents are signed with ed25519 keys for integrity proof
- **Manifest generation**: Produces JSON reports showing exact percentages of human/AI/cited content
- **Verification tools**: Standalone CLI verifier and web badge service

**Tech Stack**: Tauri (Rust), React, TypeScript, Tiptap editor, SQLite, OpenAI API, ed25519 cryptography

**Target Users**: Content creators, bloggers, researchers who want to transparently document AI assistance and combat "AI slop"

**Golden rule**: When unsure about implementation details or requirements, ALWAYS consult the developer rather than making assumptions.

---

## 1. Non-negotiable golden rules

| AI *may* do                                                            | AI *must NOT* do                                                                    |
| ----------------------------------------------------------------------|-------------------------------------------------------------------------------------|
|  Whenever unsure about something that's related to the project, ask the developer for clarification before making changes.    |  ❌ Write changes or use tools when you are not sure about something project specific, or if you don't have context for a particular feature/decision. |
|  Add/update **`AIDEV-NOTE:` anchor comments** near non-trivial edited code. | ❌ Delete or mangle existing `AIDEV-` comments.                                     |

---

## 2. Build, test & utility commands

**Development:**
```bash
npm run tauri dev          # Start development server with hot reload
npm run dev               # Frontend-only development (for UI work)
export OPENAI_API_KEY="sk-..." # Required for AI functionality
```

**Building:**
```bash
npm run tauri build       # Build production desktop app for current platform
npm run build            # Build frontend only
```

**Backend (Rust):**
```bash
cd src-tauri
cargo build              # Build Rust backend
cargo build --bin sonnun-verify  # Build standalone verifier CLI
cargo test               # Run Rust tests
```

**Database:**
```bash
# SQLite database is auto-created on first run as sonnun.db
# View events: sqlite3 sonnun.db "SELECT * FROM events ORDER BY timestamp DESC LIMIT 10"
```

**Verification:**
```bash
./target/debug/sonnun-verify signed_document.html  # Verify exported document
```

**Badge Service:**
```bash
cd badge-service
npm install && npm start  # Run verification badge service on port 3001
```

---

## 3. Coding standards

**TypeScript/React:**
- Use functional components with hooks (no class components)
- Prefer `interface` over `type` for object shapes
- Use `React.FC<Props>` for component typing
- Props interfaces should end with `Props` (e.g., `EditorPaneProps`)
- Use `const` for component definitions: `const EditorPane: React.FC<Props> = ({ ... }) => {}`

**File Naming:**
- Components: PascalCase (e.g., `EditorPane.tsx`, `ManifestModal.tsx`)
- Utils/Services: camelCase (e.g., `manifestGenerator.ts`, `cryptoService.ts`)
- Constants: UPPER_SNAKE_CASE
- Directories: kebab-case (e.g., `src/components/`, `src/utils/`)

**Imports:**
```typescript
// External libraries first
import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'

// Internal imports second, with relative paths
import { ProvenanceMark } from '../extensions/ProvenanceMark'
import CitationModal from './CitationModal'
```

**Rust Conventions:**
- Use snake_case for functions and variables
- Commands exposed to frontend should use `#[tauri::command]`
- Return `Result<T, String>` for Tauri commands (String errors are user-friendly)
- Use `serde` derives for data structures that cross the Tauri bridge

**Error Handling:**
- Frontend: Use try-catch with console.error for debugging, user-friendly alerts for errors
- Backend: Return descriptive error messages in Result::Err
- Log all provenance events even if secondary operations fail

**Security:**
- NEVER log or expose private keys
- API keys should come from environment variables only
- Validate all user inputs before database operations
- Use proper HTML escaping for user-generated content

---

## 4. Project layout & Core Components

| Directory               | Description                                       |
| ----------------------- | ------------------------------------------------- |
| `src/`                  | React frontend source code                       |
| `src/components/`       | React components (EditorPane, AssistantPanel, etc.) |
| `src/extensions/`       | Tiptap extensions (ProvenanceMark)               |
| `src/utils/`            | Utility functions (manifestGenerator, etc.)     |
| `src-tauri/`            | Rust backend source code                        |
| `src-tauri/src/bin/`    | Additional Rust binaries (verify CLI)           |
| `badge-service/`        | Node.js verification badge service               |
| `public-key-service/`   | Static files for public key distribution        |
| `keys/`                 | Generated ed25519 key pairs (git-ignored)       |

**Core Components:**

**Frontend (`src/`):**
- `App.tsx` - Main application layout and state management
- `components/EditorPane.tsx` - Tiptap editor with provenance tracking
- `components/AssistantPanel.tsx` - ChatGPT integration sidebar
- `components/CitationModal.tsx` - Modal for paste citation enforcement
- `components/ManifestModal.tsx` - Display provenance manifest
- `components/KeyManager.tsx` - Cryptographic key management UI
- `extensions/ProvenanceMark.ts` - Custom Tiptap mark for content attribution
- `utils/manifestGenerator.ts` - Calculate provenance statistics

**Backend (`src-tauri/src/`):**
- `main.rs` - Tauri app entry point and command handlers
- `bin/verify.rs` - Standalone document verification CLI

**Database Schema:**
```sql
events (
  id: INTEGER PRIMARY KEY,
  timestamp: TEXT,
  event_type: TEXT,        -- 'human', 'ai', 'cited'
  text_hash: TEXT,         -- SHA-256 of inserted text
  source: TEXT,            -- 'user', model name, or citation
  span_length: INTEGER
)
```

---

## 5. Anchor comments

Add specially formatted comments throughout the codebase, where appropriate, for yourself as inline knowledge that can be easily `grep`ped for.

### Guidelines:

- Use `AIDEV-NOTE:`, `AIDEV-TODO:`, or `AIDEV-QUESTION:` (all-caps prefix) for comments aimed at AI and developers.
- Keep them concise (≤ 120 chars).
- **Important:** Before scanning files, always first try to **locate existing anchors** `AIDEV-*` in relevant subdirectories.
- **Update relevant anchors** when modifying associated code.
- **Do not remove `AIDEV-NOTE`s** without explicit human instruction.
- Make sure to add relevant anchor comments, whenever a file or piece of code is:
  * too long, or
  * too complex, or
  * very important, or
  * confusing, or
  * could have a bug unrelated to the task you are currently working on.

Example:
```typescript
/* AIDEV-NOTE: perf-hot-path; avoid extra allocations (see ADR-24) */
async function render_feed(...) {
    // ...
}
```

---

## AI Assistant Workflow: Step-by-Step Methodology

When responding to user instructions, the AI assistant (Claude, Cursor, GPT, etc.) should follow this process to ensure clarity, correctness, and maintainability:

1. **Consult Relevant Guidance**: When the user gives an instruction, consult the relevant instructions from `CLAUDE.md` files (both root and directory-specific) for the request.
2. **Clarify Ambiguities**: Based on what you could gather, see if there's any need for clarifications. If so, ask the user targeted questions before proceeding.
3. **Break Down & Plan**: Break down the task at hand and chalk out a rough plan for carrying it out, referencing project conventions and best practices.
4. **Trivial Tasks**: If the plan/request is trivial, go ahead and get started immediately.
5. **Non-Trivial Tasks**: Otherwise, present the plan to the user for review and iterate based on their feedback.
6. **Track Progress**: Use a to-do list (internally, or optionally in a `TODOS.md` file) to keep track of your progress on multi-step or complex tasks.
7. **If Stuck, Re-plan**: If you get stuck or blocked, return to step 3 to re-evaluate and adjust your plan.
8. **Update Documentation**: Once the user's request is fulfilled, update relevant anchor comments (`AIDEV-NOTE`, etc.) and `CLAUDE.md` files in the files and directories you touched.
9. **User Review**: After completing the task, ask the user to review what you've done, and repeat the process as needed.
10. **Session Boundaries**: If the user's request isn't directly related to the current context and can be safely started in a fresh session, suggest starting from scratch to avoid context confusion.
