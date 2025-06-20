// AIDEV-NOTE: Utility functions for provenance calculations and manifest generation

export interface ProvenanceStats {
  humanPercentage: number
  aiPercentage: number
  citedPercentage: number
  totalCharacters: number
}

export interface ProvenanceEvent {
  timestamp: string
  event_type: 'human' | 'ai' | 'cited'
  text_hash: string
  source: string
  span_length: number
}

export interface ManifestData {
  human_percentage: number
  ai_percentage: number
  cited_percentage: number
  total_characters: number
  events: ProvenanceEvent[]
  generated_at: string
  document_hash: string
}

// AIDEV-NOTE: Core calculation engine for real-time provenance statistics
export function calculateProvenanceFromText(content: string): ProvenanceStats {
  // Parse HTML content and extract provenance data attributes
  const parser = new DOMParser()
  const doc = parser.parseFromString(content, 'text/html')

  let humanChars = 0
  let aiChars = 0
  let citedChars = 0

  // Walk through all text nodes and their provenance marks
  const walker = doc.createTreeWalker(doc.body, NodeFilter.SHOW_ALL, null)

  let node: Node | null
  while ((node = walker.nextNode())) {
    if (node.nodeType === Node.ELEMENT_NODE) {
      const element = node as Element
      if (element.hasAttribute('data-provenance')) {
        const text = element.textContent || ''
        const type = element.getAttribute('data-type')

        switch (type) {
          case 'human':
            humanChars += text.length
            break
          case 'ai':
            aiChars += text.length
            break
          case 'cited':
            citedChars += text.length
            break
        }
      }
    } else if (node.nodeType === Node.TEXT_NODE && node.textContent) {
      // Text without provenance marks defaults to human
      const parent = node.parentElement
      if (!parent?.hasAttribute('data-provenance')) {
        humanChars += node.textContent.length
      }
    }
  }

  const total = humanChars + aiChars + citedChars

  return {
    humanPercentage: total > 0 ? (humanChars / total) * 100 : 100,
    aiPercentage: total > 0 ? (aiChars / total) * 100 : 0,
    citedPercentage: total > 0 ? (citedChars / total) * 100 : 0,
    totalCharacters: total,
  }
}

// AIDEV-NOTE: Generates SHA-256 hash for document integrity verification
export async function hashContent(content: string): Promise<string> {
  const encoder = new TextEncoder()
  const data = encoder.encode(content)
  const hashBuffer = await crypto.subtle.digest('SHA-256', data)
  const hashArray = Array.from(new Uint8Array(hashBuffer))
  return hashArray.map((b) => b.toString(16).padStart(2, '0')).join('')
}

// AIDEV-NOTE: Compiles complete manifest with provenance events and document metadata
export async function generateCompleteManifest(
  content: string,
  events: ProvenanceEvent[]
): Promise<ManifestData> {
  const stats = calculateProvenanceFromText(content)
  const documentHash = await hashContent(content)

  return {
    human_percentage: stats.humanPercentage,
    ai_percentage: stats.aiPercentage,
    cited_percentage: stats.citedPercentage,
    total_characters: stats.totalCharacters,
    events: events.sort(
      (a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
    ),
    generated_at: new Date().toISOString(),
    document_hash: documentHash,
  }
}

// AIDEV-NOTE: Validation helpers for data integrity checks
export function validateProvenanceEvent(event: Partial<ProvenanceEvent>): string[] {
  const errors: string[] = []

  if (!event.timestamp) errors.push('Timestamp is required')
  if (!event.event_type || !['human', 'ai', 'cited'].includes(event.event_type)) {
    errors.push('Event type must be human, ai, or cited')
  }
  if (!event.source) errors.push('Source is required')
  if (typeof event.span_length !== 'number' || event.span_length < 0) {
    errors.push('Span length must be a non-negative number')
  }

  return errors
}

export function isValidManifest(manifest: any): manifest is ManifestData {
  return (
    manifest !== null &&
    typeof manifest === 'object' &&
    typeof manifest.human_percentage === 'number' &&
    typeof manifest.ai_percentage === 'number' &&
    typeof manifest.cited_percentage === 'number' &&
    typeof manifest.total_characters === 'number' &&
    Array.isArray(manifest.events) &&
    typeof manifest.generated_at === 'string' &&
    typeof manifest.document_hash === 'string'
  )
}
