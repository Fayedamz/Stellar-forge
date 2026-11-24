import React from 'react';

export type AnomalyCardProps = {
  label: string;
  unit: string;
  points: Array<{ timestamp: number; value: number }>;
  latest: number | null;
  mean: number;
  standardDeviation: number;
  lowerBound: number;
  upperBound: number;
  anomalousPoints: number;
  isAnomaly: boolean;
};

function formatValue(value: number | null, unit: string) {
  if (value == null) return '—';
  const decimals = unit === '%' || unit === 'ms' ? (unit === '%' ? 1 : 1) : 0;
  return `${value.toFixed(decimals)}${unit === '%' ? '%' : ` ${unit}`}`;
}

export default function AnomalyCard(props: AnomalyCardProps) {
  const width = 280;
  const height = 78;
  const values = props.points.map((point) => point.value);
  const min = Math.min(...values, props.lowerBound);
  const max = Math.max(...values, props.upperBound);
  const range = max - min || 1;
  const x = (index: number) => values.length === 1 ? 0 : (index / (values.length - 1)) * width;
  const y = (value: number) => height - ((value - min) / range) * height;
  const polyline = values.map((value, index) => `${x(index)},${y(value)}`).join(' ');
  const upperY = y(props.upperBound);
  const lowerY = y(props.lowerBound);

  return (
    <article className={`anomaly-card${props.isAnomaly ? ' anomaly-card--alert' : ''}`} aria-label={`${props.label} anomaly card`}>
      <header className="anomaly-card__header">
        <div>
          <span className="anomaly-card__label">{props.label}</span>
          <strong>{formatValue(props.latest, props.unit)}</strong>
        </div>
        <span className={`anomaly-card__status${props.isAnomaly ? ' anomaly-card__status--alert' : ''}`}>
          {props.isAnomaly ? `${props.anomalousPoints} spike${props.anomalousPoints === 1 ? '' : 's'}` : 'Stable'}
        </span>
      </header>
      <svg className="anomaly-card__chart" viewBox={`0 0 ${width} ${height}`} role="img" aria-label={`${props.label} current versus baseline bounds`}>
        <rect x="0" y={upperY} width={width} height={Math.max(0, lowerY - upperY)} className="anomaly-card__bounds" />
        <line x1="0" x2={width} y1={y(props.mean)} y2={y(props.mean)} className="anomaly-card__baseline" />
        <polyline points={polyline} className="anomaly-card__line" />
        {values.map((value, index) => {
          const zScore = props.standardDeviation === 0 ? 0 : (value - props.mean) / props.standardDeviation;
          return Math.abs(zScore) > 2 ? (
            <circle key={index} cx={x(index)} cy={y(value)} r="4" className="anomaly-card__spike" />
          ) : null;
        })}
      </svg>
      <footer className="anomaly-card__footer">
        <span>Baseline {formatValue(props.mean, props.unit)}</span>
        <span>±2σ {formatValue(props.upperBound, props.unit)}</span>
      </footer>
    </article>
  );
}
