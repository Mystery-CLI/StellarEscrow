import React, { useState, useEffect } from 'react';
import { fetchAnalyticsData } from './api';
import { TradeVolumeChart, TradeVolumeData } from './TradeVolumeChart';
import { SuccessRateChart, SuccessRateData } from './SuccessRateChart';

export type TimeRange = '7d' | '30d' | '90d';
export type TradeType = 'all' | 'crypto' | 'fiat';

export interface AnalyticsData {
  volume: TradeVolumeData[];
  successRate: SuccessRateData;
}

export const AnalyticsDashboard: React.FC = () => {
  const [data, setData] = useState<AnalyticsData | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [timeRange, setTimeRange] = useState<TimeRange>('30d');
  const [tradeType, setTradeType] = useState<TradeType>('all');

  useEffect(() => {
    let isMounted = true;
    setLoading(true);
    setError(null);
    
    fetchAnalyticsData(timeRange, tradeType)
      .then((res) => {
        if (isMounted) {
          setData(res);
          setLoading(false);
        }
      })
      .catch((err) => {
        if (isMounted) {
          setError(err.message || 'An error occurred while fetching data');
          setLoading(false);
        }
      });
      
    return () => { isMounted = false; };
  }, [timeRange, tradeType]);

  const handleExportCSV = () => {
    if (!data) return;
    
    // Create CSV content
    const header = 'Time,Volume\n';
    const volumeRows = data.volume.map((v: TradeVolumeData) => `${v.time},${v.volume}`).join('\n');
    const successRow = `\nSuccess Rate\nSuccess,${data.successRate.success}\nFailed,${data.successRate.failed}\n`;
    
    const csvContent = header + volumeRows + successRow;
    
    // Export functionality
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    
    link.href = url;
    link.setAttribute('download', `analytics_${timeRange}_${tradeType}.csv`);
    document.body.appendChild(link);
    link.click();
    
    // Cleanup
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  };

  return (
    <div style={{ fontFamily: 'system-ui, sans-serif', padding: '20px', width: '100%', maxWidth: '1200px', margin: '0 auto', background: '#f8fafc', borderRadius: '12px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px', flexWrap: 'wrap', gap: '15px' }}>
        <h2 style={{ margin: 0, color: '#1e293b' }}>Analytics Dashboard</h2>
        
        <div style={{ display: 'flex', gap: '10px', alignItems: 'center', flexWrap: 'wrap' }}>
          <select 
            value={timeRange} 
            onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setTimeRange(e.target.value as TimeRange)}
            style={{ padding: '8px 12px', justifySelf: 'auto', borderRadius: '8px', border: '1px solid #cbd5e1', background: 'white', minWidth: '120px' }}
          >
            <option value="7d">Last 7 Days</option>
            <option value="30d">Last 30 Days</option>
            <option value="90d">Last 90 Days</option>
          </select>

          <select 
            value={tradeType} 
            onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setTradeType(e.target.value as TradeType)}
            style={{ padding: '8px 12px', justifySelf: 'auto', borderRadius: '8px', border: '1px solid #cbd5e1', background: 'white', minWidth: '120px' }}
          >
            <option value="all">All Trades</option>
            <option value="crypto">Crypto Only</option>
            <option value="fiat">Fiat Only</option>
          </select>

          <button 
            onClick={handleExportCSV}
            disabled={!data || loading}
            style={{ 
              padding: '8px 16px', 
              background: '#3b82f6', 
              color: 'white', 
              border: 'none', 
              borderRadius: '8px',
              cursor: (!data || loading) ? 'not-allowed' : 'pointer',
              opacity: (!data || loading) ? 0.6 : 1,
              fontWeight: 500,
              minWidth: '120px'
            }}
          >
            Export Data
          </button>
        </div>
      </div>

      {loading && (
        <div style={{ padding: '40px', textAlign: 'center', color: '#64748b' }}>
          <h3>Loading analytics data...</h3>
        </div>
      )}

      {error && !loading && (
        <div style={{ padding: '20px', background: '#fee2e2', color: '#ef4444', borderRadius: '8px', textAlign: 'center' }}>
          <p>{error}</p>
          <button onClick={() => setTimeRange(timeRange)} style={{ padding: '6px 12px', marginTop: '10px' }}>Retry</button>
        </div>
      )}

      {!loading && !error && data && (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: '20px' }}>
          <div style={{ background: 'white', padding: '20px', borderRadius: '12px', border: '1px solid #e2e8f0', boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.05)', gridColumn: '1 / -1' }}>
            <TradeVolumeChart data={data.volume} />
          </div>
          <div style={{ background: 'white', padding: '20px', borderRadius: '12px', border: '1px solid #e2e8f0', boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.05)', display: 'flex', justifyContent: 'center' }}>
            <SuccessRateChart data={data.successRate} />
          </div>
        </div>
      )}
    </div>
  );
};
