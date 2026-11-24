import { describe, expect, it } from 'vitest';

import { calculateStats, evaluateAnomalySeries, buildSyntheticMetricDataset } from './anomalyModel.js';

describe('anomaly calculations', () => {
  it('calculateStats returns mean and population standard deviation', () => {
    const stats = calculateStats([10, 10, 14, 14]);
    expect(stats.mean).toBe(12);
    expect(stats.standardDeviation).toBe(2);
    expect(stats.count).toBe(4);
  });

  it('evaluateAnomalySeries flags values outside two standard deviations', () => {
    const normal = Array.from({ length: 18 }, (_, index) => ({ timestamp: index, value: 100 + (index % 3) }));
    const points = [...normal, { timestamp: 18, value: 180 }];
    const result = evaluateAnomalySeries(points);
    expect(result.isAnomaly).toBe(true);
    expect(result.latest).toBe(180);
    expect(result.zScore).toBeGreaterThan(2);
    expect(result.anomalousPoints).toBeGreaterThanOrEqual(1);
  });

  it('evaluateAnomalySeries keeps normal series stable', () => {
    const points = Array.from({ length: 24 }, (_, index) => ({ timestamp: index, value: 50 + (index % 2) }));
    const result = evaluateAnomalySeries(points);
    expect(result.isAnomaly).toBe(false);
  });

  it('synthetic dataset includes artificial spikes and contains all required metrics', () => {
    const dataset = buildSyntheticMetricDataset();
    expect(dataset.length).toBe(6);
    expect(dataset.every((metric) => metric.points.length === 24)).toBe(true);
    expect(dataset.some((metric) => evaluateAnomalySeries(metric.points).isAnomaly)).toBe(true);
  });
});
