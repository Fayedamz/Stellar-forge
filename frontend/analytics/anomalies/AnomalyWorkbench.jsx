import React, { useMemo, useState } from 'react';
import AnomalyCard from '../../components/anomaly_card.tsx';
import { buildSyntheticMetricDataset, evaluateAnomalySeries } from './anomalyModel.js';

const RANGES = [
  { label: '1 hour', minutes: 60 },
  { label: '6 hours', minutes: 360 },
  { label: '24 hours', minutes: 1440 },
];

export default function AnomalyWorkbench() {
  const [range, setRange] = useState(RANGES[2].minutes);
  const dataset = useMemo(() => buildSyntheticMetricDataset(), []);

  const cards = useMemo(() => {
    const cutoff = Date.now() - range * 60 * 1000;
    return dataset.map((metric) => {
      const points = metric.points.filter((point) => point.timestamp >= cutoff);
      return { metric, points, result: evaluateAnomalySeries(points) };
    });
  }, [dataset, range]);

  const anomalyCount = cards.filter(({ result }) => result.isAnomaly).length;

  return (
    <section className="anomaly-workbench" aria-label="Cluster metric anomaly detection workbench">
      <div className="anomaly-toolbar">
        <div>
          <span className="eyebrow">CLUSTER SIGNALS</span>
          <h2>Anomaly detection</h2>
        </div>
        <div className="anomaly-toolbar__controls">
          <span className={`anomaly-summary${anomalyCount ? ' anomaly-summary--alert' : ''}`}>
            {anomalyCount} anomalous {anomalyCount === 1 ? 'series' : 'series'}
          </span>
          <label className="select-wrap">
            Time range
            <select value={range} onChange={(event) => setRange(Number(event.target.value))}>
              {RANGES.map((option) => (
                <option key={option.minutes} value={option.minutes}>{option.label}</option>
              ))}
            </select>
          </label>
        </div>
      </div>

      <div className="anomaly-grid">
        {cards.map(({ metric, points, result }) => (
          <AnomalyCard
            key={metric.id}
            label={metric.label}
            unit={metric.unit}
            points={points}
            latest={result.latest}
            mean={result.mean}
            standardDeviation={result.standardDeviation}
            lowerBound={result.lowerBound}
            upperBound={result.upperBound}
            anomalousPoints={result.anomalousPoints}
            isAnomaly={result.isAnomaly}
          />
        ))}
      </div>
    </section>
  );
}
