import React from 'react'

// AIDEV-NOTE: Minimal provenance display with typography indicators
interface ProvenanceLegendProps {
  stats?: {
    humanPercentage: number
    aiPercentage: number
    citedPercentage: number
  }
  className?: string
}

const ProvenanceLegend: React.FC<ProvenanceLegendProps> = ({ stats, className = '' }) => {
  if (!stats) return null

  return (
    <div className={`provenance-legend ${className}`}>
      <div className="legend-items">
        <div className="legend-item">
          <span className="legend-label">Human</span>
          <span className="legend-percentage">{Math.round(stats.humanPercentage)}%</span>
        </div>
        <div className="legend-item">
          <span className="legend-label">AI</span>
          <span className="legend-percentage">{Math.round(stats.aiPercentage)}%</span>
        </div>
        <div className="legend-item">
          <span className="legend-label">Cited</span>
          <span className="legend-percentage">{Math.round(stats.citedPercentage)}%</span>
        </div>
      </div>
    </div>
  )
}

export default ProvenanceLegend