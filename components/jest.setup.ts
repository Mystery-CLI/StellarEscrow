import React from 'react';
import { toHaveNoViolations } from 'jest-axe';
require('@testing-library/jest-dom');

// Extend expect with jest-axe matchers
expect.extend(toHaveNoViolations as any);

// Mock Chart.js
jest.mock('chart.js', () => ({
  Chart: {
    register: jest.fn(),
  },
  CategoryScale: jest.fn(),
  LinearScale: jest.fn(),
  PointElement: jest.fn(),
  LineElement: jest.fn(),
  Title: jest.fn(),
  Tooltip: jest.fn(),
  Legend: jest.fn(),
  Filler: jest.fn(),
  ArcElement: jest.fn(),
}));

// Mock react-chartjs-2
jest.mock('react-chartjs-2', () => ({
  Line: ({ data, options }: any) => React.createElement('div', { 'data-testid': 'line-chart', 'data-data': JSON.stringify(data), 'data-options': JSON.stringify(options) }),
  Doughnut: ({ data, options }: any) => React.createElement('div', { 'data-testid': 'doughnut-chart', 'data-data': JSON.stringify(data), 'data-options': JSON.stringify(options) }),
}));
