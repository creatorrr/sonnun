import { Mark, mergeAttributes } from '@tiptap/core'

export interface ProvenanceMarkOptions {
  HTMLAttributes: Record<string, any>
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    provenanceMark: {
      setProvenanceMark: (attributes?: { source: string; type: string }) => ReturnType
      toggleProvenanceMark: (attributes?: { source: string; type: string }) => ReturnType
      unsetProvenanceMark: () => ReturnType
    }
  }
}

// AIDEV-NOTE: Core extension - renders invisible provenance metadata as data attributes on spans
// AIDEV-QUESTION: Should we add timestamp to provenance data for audit trails?
export const ProvenanceMark = Mark.create<ProvenanceMarkOptions>({
  name: 'provenanceMark',

  addOptions() {
    return {
      HTMLAttributes: {},
    }
  },

  addAttributes() {
    return {
      source: {
        default: 'user',
        parseHTML: element => element.getAttribute('data-source'),
        renderHTML: attributes => {
          if (!attributes.source) {
            return {}
          }
          return {
            'data-source': attributes.source,
          }
        },
      },
      type: {
        default: 'human',
        // AIDEV-TODO: Add validation to ensure type is one of: 'human', 'ai', 'cited'
        parseHTML: element => element.getAttribute('data-type'),
        renderHTML: attributes => {
          if (!attributes.type) {
            return {}
          }
          return {
            'data-type': attributes.type,
          }
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

  renderHTML({ HTMLAttributes }) {
    // AIDEV-NOTE: HTML5 data attributes enable CSS styling and JS access
    return [
      'span',
      mergeAttributes(this.options.HTMLAttributes, HTMLAttributes, {
        'data-provenance': true,
        class: `provenance-${HTMLAttributes.type || 'human'}`,
      }),
      0,
    ]
  },

  addCommands() {
    return {
      setProvenanceMark:
        (attributes) =>
        ({ commands }) => {
          return commands.setMark(this.name, attributes)
        },
      toggleProvenanceMark:
        (attributes) =>
        ({ commands }) => {
          return commands.toggleMark(this.name, attributes)
        },
      unsetProvenanceMark:
        () =>
        ({ commands }) => {
          return commands.unsetMark(this.name)
        },
    }
  },
})