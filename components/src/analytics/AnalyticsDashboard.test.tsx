import React from 'react';
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { AnalyticsDashboard } from './AnalyticsDashboard';

// Mock the fetch function
jest.mock('./api', () => ({
  fetchAnalyticsData: jest.fn(),
}));

const { fetchAnalyticsData } = require('./api');

beforeEach(() => {
  (fetchAnalyticsData as jest.MockedFunction<typeof fetchAnalyticsData>).mockResolvedValue({
    volume: Array.from({ length: 7 }).map((_, i) => ({
      time: `Day ${i + 1}`,
      volume: Math.floor(Math.random() * 10000) + 1000,
    })),
    successRate: {
      success: Math.floor((80 + Math.random() * 15)),
      failed: Math.floor((5 + Math.random() * 15)),
    },
  });
});

describe('AnalyticsDashboard', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders loading state initially', () => {
    render(<AnalyticsDashboard />);
    expect(screen.getByText('Loading analytics data...')).toBeInTheDocument();
  });

  it('renders dashboard title', () => {
    render(<AnalyticsDashboard />);
    expect(screen.getByText('Analytics Dashboard')).toBeInTheDocument();
  });

  it('renders time range selector', () => {
    render(<AnalyticsDashboard />);
    expect(screen.getByDisplayValue('Last 30 Days')).toBeInTheDocument();
  });

  it('renders trade type selector', () => {
    render(<AnalyticsDashboard />);
    expect(screen.getByDisplayValue('All Trades')).toBeInTheDocument();
  });

  it('renders export button', () => {
    render(<AnalyticsDashboard />);
    expect(screen.getByText('Export Data')).toBeInTheDocument();
  });

  it('export button is disabled during loading', () => {
    render(<AnalyticsDashboard />);
    const exportButton = screen.getByText('Export Data');
    expect(exportButton).toBeDisabled();
  });

  it('changes time range and refetches data', async () => {
    render(<AnalyticsDashboard />);

    const select = screen.getByDisplayValue('Last 30 Days');
    await act(async () => {
      await userEvent.selectOptions(select, '7d');
    });

    // Wait for loading to complete
    await waitFor(() => {
      expect(screen.queryByText('Loading analytics data...')).not.toBeInTheDocument();
    });

    // Should have charts rendered
    expect(screen.getByTestId('trade-volume-chart')).toBeInTheDocument();
    expect(screen.getByTestId('success-rate-chart')).toBeInTheDocument();
  });

  it('changes trade type and refetches data', async () => {
    render(<AnalyticsDashboard />);

    const select = screen.getByDisplayValue('All Trades');
    await act(async () => {
      await userEvent.selectOptions(select, 'crypto');
    });

    await waitFor(() => {
      expect(screen.queryByText('Loading analytics data...')).not.toBeInTheDocument();
    });

    expect(screen.getByTestId('trade-volume-chart')).toBeInTheDocument();
    expect(screen.getByTestId('success-rate-chart')).toBeInTheDocument();
  });

  it('handles fetch error', async () => {
    (fetchAnalyticsData as jest.MockedFunction<typeof fetchAnalyticsData>).mockRejectedValueOnce(new Error('Network error'));

    render(<AnalyticsDashboard />);

    await waitFor(() => {
      expect(screen.getByText('Network error')).toBeInTheDocument();
    });
  });

  it('renders charts when data is loaded', async () => {
    render(<AnalyticsDashboard />);

    await waitFor(() => {
      expect(screen.getByTestId('trade-volume-chart')).toBeInTheDocument();
      expect(screen.getByTestId('success-rate-chart')).toBeInTheDocument();
    });
  });
});