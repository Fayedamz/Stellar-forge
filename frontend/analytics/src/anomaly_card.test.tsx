import React from 'react';
import { describe, expect, it } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import AnomalyCard from '../../components/anomaly_card.tsx';

const points = Array.from({ length: 24 }, (_, index) => ({ timestamp: index, value: index === 22 ? 180 : 40 + (index % 3) }));

describe('AnomalyCard', () => {
  it('renders an anomalous metric card with an alert status', () => {
    const markup = renderToStaticMarkup(<AnomalyCard label="P99 latency" unit="ms" points={points} latest={180} mean={41} standardDeviation={4} lowerBound={33} upperBound={49} anomalousPoints={1} isAnomaly />);
    expect(markup).toContain('P99 latency');
    expect(markup).toContain('anomaly-card--alert');
    expect(markup).toContain('1 spike');
  });

  it('renders a stable card without an alert class', () => {
    const markup = renderToStaticMarkup(<AnomalyCard label="CPU usage" unit="%" points={points.map((point) => ({ ...point, value: 55 }))} latest={55} mean={55} standardDeviation={1} lowerBound={53} upperBound={57} anomalousPoints={0} isAnomaly={false} />);
    expect(markup).toContain('CPU usage');
    expect(markup).toContain('Stable');
    expect(markup).not.toContain('anomaly-card--alert');
  });
});
