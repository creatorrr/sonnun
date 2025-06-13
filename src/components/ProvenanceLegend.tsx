import React from 'react'

interface ProvenanceLegendProps {
  stats?: {
    humanPercentage: number
    aiPercentage: number
    citedPercentage: number
  }
  className?: string
}

interface LegendItem {
  type: 'human' | 'ai' | 'cited'
  label: string
  color: string
  description: string
}

const ProvenanceLegend: React.FC<ProvenanceLegendProps> = ({ stats, className = '' }) => {
  // AIDEV-NOTE: Design consistency - these colors must sync with .provenance-{type} CSS classes
  const legendItems: LegendItem[] = [
    {
      type: 'human',
      label: 'Human Written',
      color: '#4CAF50',
      description: 'Content written by the document author',
    },
    {
      type: 'ai',
      label: 'AI Assisted',
      color: '#2196F3',
      description: 'Content generated with AI assistance',
    },
    {
      type: 'cited',
      label: 'Cited/Quoted',
      color: '#FF9800',
      description: 'Content from external sources',
    },
  ]

  return (
    <div className={`provenance-legend ${className}`}>
      <h3>Content Provenance</h3>
      <div className="legend-items">
        {legendItems.map((item) => (
          <div key={item.type} className="legend-item">
            <div className="legend-color-wrapper">
              <span
                className="legend-color"
                style={{ backgroundColor: item.color }}
                aria-hidden="true"
              />
              <span className="legend-label">{item.label}</span>
              {stats && (
                <span className="legend-percentage">
                  {Math.round(
                    item.type === 'human'
                      ? stats.humanPercentage
                      : item.type === 'ai'
                      ? stats.aiPercentage
                      : stats.citedPercentage
                  )}%
                </span>
              )}
            </div>
            <p className="legend-description">{item.description}</p>
          </div>
        ))}
      </div>
      
      {stats && (
        <div className="legend-bar">
          <div
            className="bar-segment human"
            style={{ width: `${stats.humanPercentage}%` }}
            title={`Human: ${Math.round(stats.humanPercentage)}%`}
          />
          <div
            className="bar-segment ai"
            style={{ width: `${stats.aiPercentage}%` }}
            title={`AI: ${Math.round(stats.aiPercentage)}%`}
          />
          <div
            className="bar-segment cited"
            style={{ width: `${stats.citedPercentage}%` }}
            title={`Cited: ${Math.round(stats.citedPercentage)}%`}
          />
        </div>
      )}
    </div>
  )
}

export default ProvenanceLegend