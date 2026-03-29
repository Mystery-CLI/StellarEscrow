import React from 'react';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip, // Ensure correct exports are mocked or available
  Legend,
  Filler
} from 'chart.js';
import { Line } from 'react-chartjs-2';

ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  Filler
);

export interface TradeVolumeData {
  time: string;
  volume: number;
}

export interface TradeVolumeChartProps {
  data: TradeVolumeData[];
}

export const TradeVolumeChart: React.FC<TradeVolumeChartProps> = ({ data }) => {
  const chartData = {
    labels: data.map((d) => d.time),
    datasets: [
      {
        label: 'Trade Volume',
        data: data.map((d) => d.volume),
        borderColor: 'rgb(75, 192, 192)',
        backgroundColor: 'rgba(75, 192, 192, 0.2)',
        tension: 0.4,
        fill: true,
      },
    ],
  };

  const options = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: {
        position: 'top' as const,
      },
      title: {
        display: true,
        text: 'Trade Volume Over Time',
      },
    },
    scales: {
      y: {
        beginAtZero: true,
        title: {
          display: true,
          text: 'Volume ($)',
        },
      },
      x: {
        title: {
          display: true,
          text: 'Time Frame',
        },
      },
    },
  };

  return (
    <div style={{ position: 'relative', height: '300px', width: '100%', padding: '10px' }}>
      <Line data={chartData} options={options} />
    </div>
  );
};
