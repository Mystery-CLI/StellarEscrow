// Mock API Call
import { AnalyticsData } from './AnalyticsDashboard';

export const fetchAnalyticsData = async (timeRange: string, tradeType: string): Promise<AnalyticsData> => {
  const multiplier = timeRange === '7d' ? 1 : timeRange === '30d' ? 4 : 12;
  const typeMultiplier = tradeType === 'all' ? 1 : tradeType === 'crypto' ? 0.7 : 0.3;
  
  const mockData: AnalyticsData = {
    volume: Array.from({ length: multiplier * 7 }).map((_, i) => ({
      time: `Day ${i + 1}`,
      volume: Math.floor(Math.random() * 10000 * typeMultiplier) + 1000,
    })),
    successRate: {
      success: Math.floor((80 + Math.random() * 15) * typeMultiplier * multiplier),
      failed: Math.floor((5 + Math.random() * 15) * typeMultiplier * multiplier),
    },
  };
  
  return mockData;
};