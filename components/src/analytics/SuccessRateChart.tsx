import React from 'react';
import { Chart as ChartJS, ArcElement, Tooltip, Legend } from 'chart.js';
import { Doughnut } from 'react-chartjs-2';

ChartJS.register(ArcElement, Tooltip, Legend);

export interface SuccessRateData {
  success: number;
  failed: number;
}

export interface SuccessRateChartProps {
  data: SuccessRateData;
}

export const SuccessRateChart: React.FC<SuccessRateChartProps> = ({ data }) => {
  const total = data.success + data.failed;
  const successPercent = total > 0 ? ((data.success / total) * 100).toFixed(1) : '0.0';
  const failedPercent = total > 0 ? ((data.failed / total) * 100).toFixed(1) : '0.0';

  const chartData = {
    labels: [`Successful (${successPercent}%)`, `Failed (${failedPercent}%)`],
    datasets: [
      {
        label: 'Trades',
        data: [data.success, data.failed],
        backgroundColor: [
          'rgba(54, 162, 235, 0.6)',
          'rgba(255, 99, 132, 0.6)',
        ],
        borderColor: [
          'rgba(54, 162, 235, 1)',
          'rgba(255, 99, 132, 1)',
        ],
        borderWidth: 1,
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
        text: 'Trade Success Rate',
      },
    },
  };

  return (
    <div style={{ position: 'relative', height: '300px', width: '100%', padding: '10px' }}>
      <Doughnut data={chartData} options={options} />
    </div>
  );
};
