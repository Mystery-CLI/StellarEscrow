import React from 'react';
import { render, screen } from '@testing-library/react';
import { SuccessRateChart } from './SuccessRateChart';

describe('SuccessRateChart', () => {
  const mockData = {
    success: 80,
    failed: 20,
  };

  it('renders chart with data', () => {
    render(<SuccessRateChart data={mockData} />);
    const chart = screen.getByTestId('doughnut-chart');
    expect(chart).toBeInTheDocument();
  });

  it('calculates percentages correctly', () => {
    render(<SuccessRateChart data={mockData} />);
    const chart = screen.getByTestId('doughnut-chart');
    const data = JSON.parse(chart.getAttribute('data-data') || '{}');
    expect(data.labels).toEqual(['Successful (80.0%)', 'Failed (20.0%)']);
    expect(data.datasets[0].data).toEqual([80, 20]);
  });

  it('handles zero total correctly', () => {
    const zeroData = { success: 0, failed: 0 };
    render(<SuccessRateChart data={zeroData} />);
    const chart = screen.getByTestId('doughnut-chart');
    const data = JSON.parse(chart.getAttribute('data-data') || '{}');
    expect(data.labels).toEqual(['Successful (0.0%)', 'Failed (0.0%)']);
  });

  it('matches snapshot', () => {
    const { container } = render(<SuccessRateChart data={mockData} />);
    expect(container.firstChild).toMatchSnapshot();
  });
});