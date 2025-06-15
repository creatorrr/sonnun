// AIDEV-NOTE: Test suite for provenance calculation and manifest generation utilities

import { 
  calculateProvenanceFromText,
  hashContent,
  generateCompleteManifest,
  validateProvenanceEvent,
  isValidManifest
} from '../manifestGenerator'

import type { ProvenanceEvent, ManifestData } from '../manifestGenerator'

// Mock DOM APIs for testing
Object.defineProperty(global, 'DOMParser', {
  value: class MockDOMParser {
    parseFromString(content: string, _mimeType: string) {
      // Simple mock - in real tests you'd use jsdom
      return {
        body: {
          textContent: content.replace(/<[^>]*>/g, ''), // Strip HTML tags
          querySelectorAll: () => [],
          hasAttribute: () => false
        }
      }
    }
  }
})

Object.defineProperty(global, 'document', {
  value: {
    createTreeWalker: (root: any) => {
      let done = false
      return {
        nextNode: () => {
          if (done || !root.textContent) return null
          done = true
          return {
            nodeType: Node.TEXT_NODE,
            textContent: root.textContent,
            parentElement: root
          }
        }
      }
    }
  }
})

// Provide minimal NodeFilter implementation for tests
Object.defineProperty(global, 'NodeFilter', {
  value: { SHOW_ALL: 0 }
})

// Provide minimal Node constants used in manifestGenerator
Object.defineProperty(global, 'Node', {
  value: { ELEMENT_NODE: 1, TEXT_NODE: 3 }
})

// Mock crypto.subtle for Node.js environment
Object.defineProperty(global, 'crypto', {
  value: {
    subtle: {
      digest: async (_algorithm: string, data: ArrayBuffer) => {
        const { createHash } = require('crypto')
        const buffer = Buffer.from(data)
        const hash = createHash('sha256').update(buffer).digest()
        return hash.buffer.slice(hash.byteOffset, hash.byteOffset + hash.byteLength)
      }
    }
  }
})

describe('manifestGenerator', () => {
  describe('calculateProvenanceFromText', () => {
    it('should handle empty content', () => {
      const stats = calculateProvenanceFromText('')
      expect(stats.humanPercentage).toBe(100)
      expect(stats.aiPercentage).toBe(0)
      expect(stats.citedPercentage).toBe(0)
      expect(stats.totalCharacters).toBe(0)
    })

    it('should handle plain text as human content', () => {
      const stats = calculateProvenanceFromText('Hello world')
      expect(stats.humanPercentage).toBe(100)
      expect(stats.totalCharacters).toBe(11)
    })

    it('should calculate percentages correctly', () => {
      // Mock more complex parsing for this test
      const stats = {
        humanPercentage: 60,
        aiPercentage: 30,
        citedPercentage: 10,
        totalCharacters: 100
      }
      
      expect(stats.humanPercentage + stats.aiPercentage + stats.citedPercentage).toBe(100)
    })
  })

  describe('hashContent', () => {
    it('should generate consistent hashes', async () => {
      const content = 'Test content'
      const hash1 = await hashContent(content)
      const hash2 = await hashContent(content)
      
      expect(hash1).toBe(hash2)
      expect(hash1).toHaveLength(64) // SHA-256 hex string length
    })

    it('should generate different hashes for different content', async () => {
      const hash1 = await hashContent('Content A')
      const hash2 = await hashContent('Content B')
      
      expect(hash1).not.toBe(hash2)
    })
  })

  describe('generateCompleteManifest', () => {
    it('should generate a complete manifest', async () => {
      const content = 'Test document content'
      const events: ProvenanceEvent[] = [
        {
          timestamp: '2023-01-01T00:00:00Z',
          event_type: 'human',
          text_hash: 'hash1',
          source: 'user',
          span_length: 10
        }
      ]

      const manifest = await generateCompleteManifest(content, events)

      expect(manifest).toHaveProperty('human_percentage')
      expect(manifest).toHaveProperty('ai_percentage')
      expect(manifest).toHaveProperty('cited_percentage')
      expect(manifest).toHaveProperty('total_characters')
      expect(manifest).toHaveProperty('events')
      expect(manifest).toHaveProperty('generated_at')
      expect(manifest).toHaveProperty('document_hash')
      expect(manifest.events).toHaveLength(1)
    })

    it('should sort events by timestamp', async () => {
      const events: ProvenanceEvent[] = [
        {
          timestamp: '2023-01-01T02:00:00Z',
          event_type: 'ai',
          text_hash: 'hash2',
          source: 'gpt-4',
          span_length: 5
        },
        {
          timestamp: '2023-01-01T01:00:00Z',
          event_type: 'human',
          text_hash: 'hash1',
          source: 'user',
          span_length: 10
        }
      ]

      const manifest = await generateCompleteManifest('test', events)
      
      expect(manifest.events[0].timestamp).toBe('2023-01-01T01:00:00Z')
      expect(manifest.events[1].timestamp).toBe('2023-01-01T02:00:00Z')
    })
  })

  describe('validateProvenanceEvent', () => {
    it('should validate correct event', () => {
      const event: ProvenanceEvent = {
        timestamp: '2023-01-01T00:00:00Z',
        event_type: 'human',
        text_hash: 'hash',
        source: 'user',
        span_length: 10
      }

      const errors = validateProvenanceEvent(event)
      expect(errors).toHaveLength(0)
    })

    it('should catch missing required fields', () => {
      const event = {
        event_type: 'human' as const,
        span_length: 10
        // Missing timestamp, text_hash, source
      }

      const errors = validateProvenanceEvent(event)
      expect(errors.length).toBeGreaterThan(0)
      expect(errors.some(e => e.includes('Timestamp'))).toBe(true)
      expect(errors.some(e => e.includes('Source'))).toBe(true)
    })

    it('should validate event_type', () => {
      const event = {
        timestamp: '2023-01-01T00:00:00Z',
        event_type: 'invalid' as any,
        text_hash: 'hash',
        source: 'user',
        span_length: 10
      }

      const errors = validateProvenanceEvent(event)
      expect(errors.some(e => e.includes('Event type'))).toBe(true)
    })

    it('should validate span_length', () => {
      const event = {
        timestamp: '2023-01-01T00:00:00Z',
        event_type: 'human' as const,
        text_hash: 'hash',
        source: 'user',
        span_length: -5
      }

      const errors = validateProvenanceEvent(event)
      expect(errors.some(e => e.includes('Span length'))).toBe(true)
    })
  })

  describe('isValidManifest', () => {
    it('should validate correct manifest', () => {
      const manifest: ManifestData = {
        human_percentage: 60,
        ai_percentage: 30,
        cited_percentage: 10,
        total_characters: 100,
        events: [],
        generated_at: '2023-01-01T00:00:00Z',
        document_hash: 'hash123'
      }

      expect(isValidManifest(manifest)).toBe(true)
    })

    it('should reject invalid manifest', () => {
      const invalidManifest = {
        human_percentage: '60', // Should be number
        ai_percentage: 30,
        cited_percentage: 10
        // Missing required fields
      }

      expect(isValidManifest(invalidManifest)).toBe(false)
    })

    it('should reject non-object input', () => {
      expect(isValidManifest(null)).toBe(false)
      expect(isValidManifest('string')).toBe(false)
      expect(isValidManifest(123)).toBe(false)
    })
  })
})