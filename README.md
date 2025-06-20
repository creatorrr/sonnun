# Sonnun: Provably Honest Content Creation

> A desktop markdown editor that cryptographically proves content provenance - distinguishing human
> writing from AI assistance and external sources.

![Sonnun Logo](https://via.placeholder.com/800x200/007bff/ffffff?text=Sonnun+%E2%80%A2+Provably+Honest+Content)

## 🎯 Mission

Combat "AI slop" by making honest attribution easier than deception. Writers get a tool that
transparently tracks content origins; readers get cryptographic proof of authorship.

## ✨ Key Features

- **🔍 Real-time Provenance Tracking** - Every text insertion logged with source attribution
- **📋 Citation Enforcement** - Paste operations require source attribution via modal dialogs
- **👁️ Visual Indicators** - Different content types visually marked in the editor
- **🔐 Cryptographic Signing** - Documents signed with ed25519 keys for integrity proof
- **📊 Manifest Generation** - JSON reports showing exact percentages of human/AI/cited content
- **✅ Verification Tools** - Standalone CLI verifier and web badge service

## 🏗️ Architecture

**Trust Chain:**

```
author laptop ──signs──► markdown.html + c2pa.json
                 ▲                         │
                 │                         ▼
          private key (ed25519)   public key in profile bio
```

**Tech Stack:**

- **Frontend**: React, TypeScript, Tiptap editor
- **Backend**: Rust, Tauri framework
- **Database**: SQLite with append-only event log
- **AI**: OpenAI API integration
- **Crypto**: ed25519 signatures, C2PA standard

## 🚀 Quick Start

### Prerequisites

- [Node.js](https://nodejs.org/) (latest LTS)
- [Rust](https://rustup.rs/) (latest stable)
- OpenAI API key

### Setup

```bash
# Clone the repository
git clone https://github.com/your-org/sonnun.git
cd sonnun

# Install dependencies
npm install

# Set up environment
export OPENAI_API_KEY="your-api-key-here"

# Start development server
npm run tauri dev
```

### First Run

1. Generate cryptographic keys via the Key Manager in the sidebar
2. Copy your public key to your profile bio: `prov-pk: ed25519:MCowBQYDK2VwAyEA...`
3. Start writing! The app will track all content sources automatically

## 📖 Documentation

- **[Implementation Guide](IMPLEMENTATION.md)** - Detailed step-by-step development plan
- **[Project Plan](PROJECT-PLAN.md)** - High-level architecture and requirements
- **[Developer Guide](CLAUDE.md)** - Coding standards and AI assistant workflow

## 🔧 Development Commands

```bash
# Development
npm run tauri dev          # Start dev server with hot reload
npm run dev               # Frontend-only development

# Building
npm run tauri build       # Build desktop app for current platform
npm run build            # Build frontend only

# Backend (Rust)
cd src-tauri
cargo build              # Build Rust backend
cargo build --bin sonnun-verify  # Build CLI verifier
cargo test               # Run tests

# Verification
./target/debug/sonnun-verify document.html  # Verify signed document
```

## 🌟 Usage Example

1. **Write Content**: Type naturally in the editor - all human input is tracked
2. **Use AI Assistant**: Ask ChatGPT for help via the sidebar - AI content is marked automatically
3. **Cite External Sources**: Paste content triggers citation modal - sources are linked and tracked
4. **Generate Manifest**: Click "Publish" to see provenance breakdown
5. **Export & Sign**: Generate cryptographically signed HTML with embedded proof
6. **Verify**: Use CLI tool or web service to verify document authenticity

## 🔐 Content Attribution

All text is categorized and tracked:

```typescript
if (event.agent === 'ai') ai_tokens += span_length
else if (event.agent === 'cited') cited_tokens += span_length
else human_tokens += span_length
```

**Example Manifest:**

```json
{
  "human_percentage": "70.2%",
  "ai_percentage": "20.1%",
  "cited_percentage": "9.7%",
  "total_tokens": 1247,
  "signature": "cryptographic_proof...",
  "public_key": "ed25519:MCowBQYDK2VwAyEA..."
}
```

### Programmatic Manifest Generation

```typescript
import { generateCompleteManifest } from './src/utils/manifestGenerator'

const manifest = await generateCompleteManifest(html, events)
console.log(manifest)
```

## 🛡️ Verification

**For Readers:**

```bash
curl -s https://prove.dev/verify?url=https://blog.com/post.html | jq .result
```

**Public Key Distribution:**

- Profile bio: `prov-pk: ed25519:MCowBQYDK2VwAyEA...`
- Webfinger: `/.well-known/prov.json`
- Badge verification with embedded pubkey hash

## 🗺️ Roadmap

- [x] **Phase 1**: Basic editor with AI integration
- [x] **Phase 2**: Provenance tracking and citation enforcement
- [x] **Phase 3**: Manifest generation and visual indicators
- [x] **Phase 4**: Cryptographic signing and export
- [ ] **Phase 5**: Standalone verification tools
- [ ] **Phase 6**: Web badge service and distribution
- [ ] **Phase 7**: Editor plugins (Obsidian, VS Code, Ghost)

## 🤝 Contributing

We welcome contributions! Please see our [development guide](CLAUDE.md) for coding standards and
workflow.

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add amazing feature'`
4. Push to branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Tauri](https://tauri.app/) for the excellent desktop app framework
- [Tiptap](https://tiptap.dev/) for the extensible rich text editor
- [OpenAI](https://openai.com/) for AI integration capabilities
- The content provenance research community

---

**"Making honesty cheaper than fraud, one document at a time."**
