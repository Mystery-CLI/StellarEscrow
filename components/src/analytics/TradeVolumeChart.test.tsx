import React from 'react';
import { render, screen } from '@testing-library/react';
import { TradeVolumeChart } from './TradeVolumeChart';

describe('TradeVolumeChart', () => {
  const mockData = [
    { time: 'Day 1', volume: 1000 },
    { time: 'Day 2', volume: 1500 },
    { time: 'Day 3', volume: 1200 },
  ];

  it('renders chart with data', () => {
    render(<TradeVolumeChart data={mockData} />);
    const chart = screen.getByTestId('line-chart');
    expect(chart).toBeInTheDocument();
  });

  it('passes correct data to chart', () => {
    render(<TradeVolumeChart data={mockData} />);
    const chart = screen.getByTestId('line-chart');
    const data = JSON.parse(chart.getAttribute('data-data') || '{}');
    expect(data.labels).toEqual(['Day 1', 'Day 2', 'Day 3']);
    expect(data.datasets[0].data).toEqual([1000, 1500, 1200]);
    expect(data.datasets[0].label).toBe('Trade Volume');
  });

  it('renders with empty data', () => {
    render(<TradeVolumeChart data={[]} />);
    const chart = screen.getByTestId('line-chart');
    expect(chart).toBeInTheDocument();
  });

  it('matches snapshot', () => {
    const { container } = render(<TradeVolumeChart data={mockData} />);
    expect(container.firstChild).toMatchSnapshot();
  });
});