export function calculateStats(values) {
  const finite = values.filter((value) => Number.isFinite(value));
  if (finite.length === 0) return { mean: 0, standardDeviation: 0, count: 0 };

  let mean = 0;
  for (const value of finite) mean += value;
  mean /= finite.length;

  let variance = 0;
  for (const value of finite) variance += (value - mean) ** 2;

  return {
    mean,
    standardDeviation: Math.sqrt(variance / finite.length),
    count: finite.length,
  };
}

export function evaluateAnomalySeries(series, threshold = 2) {
  if (!series?.length) {
    return { isAnomaly: false, latest: null, zScore: 0, mean: 0, standardDeviation: 0, lowerBound: 0, upperBound: 0, anomalousPoints: 0 };
  }

  const split = Math.max(1, Math.floor(series.length * 0.75));
  const baselineValues = series.slice(0, split).map((point) => point.value);
  const stats = calculateStats(baselineValues);
  const latest = series.at(-1)?.value ?? null;
  const deviation = stats.standardDeviation;
  const zScore = latest == null || deviation === 0 ? 0 : (latest - stats.mean) / deviation;
  let anomalousPoints = 0;

  for (const point of series) {
    const pointZ = deviation === 0 ? 0 : (point.value - stats.mean) / deviation;
    if (Math.abs(pointZ) > threshold) anomalousPoints += 1;
  }

  return {
    isAnomaly: anomalousPoints > 0,
    latest,
    zScore,
    mean: stats.mean,
    standardDeviation: deviation,
    lowerBound: stats.mean - threshold * deviation,
    upperBound: stats.mean + threshold * deviation,
    anomalousPoints,
  };
}

export function buildSyntheticMetricDataset(seed = 127) {
  let state = seed >>> 0;
  const random = () => {
    state = (1664525 * state + 1013904223) >>> 0;
    return state / 4294967296;
  };

  const makeSeries = (label, unit, base, noise, decimals, spikeAt) => {
    const points = [];
    const now = Date.now();
    for (let i = 0; i < 24; i += 1) {
      const spike = i === spikeAt ? base * (label === 'P99 latency' ? 1.8 : 2.1) : 0;
      const jitter = (random() - 0.5) * noise * 2;
      points.push({
        timestamp: now - (23 - i) * 5 * 60 * 1000,
        value: Number(Math.max(0, base + jitter + spike).toFixed(decimals)),
      });
    }
    return { id: label.toLowerCase().replace(/[^a-z0-9]+/g, '-'), label, unit, points };
  };

  return [
    makeSeries('P99 latency', 'ms', 42, 4, 1, 22),
    makeSeries('CPU usage', '%', 58, 5, 1, -1),
    makeSeries('Memory usage', '%', 68, 3, 1, 20),
    makeSeries('Error rate', '%', 0.8, 0.12, 2, 21),
    makeSeries('Ledger close time', 'ms', 5.1, 0.35, 2, -1),
    makeSeries('Consensus TPS', 'tx/s', 1280, 70, 0, 23),
  ];
}
