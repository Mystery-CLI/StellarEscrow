import axios, { AxiosInstance } from 'axios';

const BASE_URL = process.env.API_BASE_URL || 'http://localhost:3000';

export const api: AxiosInstance = axios.create({
  baseURL: BASE_URL,
  timeout: 10_000,
  headers: {
    'Content-Type': 'application/json',
    ...(process.env.API_KEY ? { 'x-api-key': process.env.API_KEY } : {}),
  },
});

// Valid Stellar test addresses
export const TEST_ADDRESSES = {
  seller: 'GBM36FA7SJUDGNIH2R4LOVQPCZETW5YXKBM36FA7SJUDGNIH2R4LOVQP',
  buyer:  'GGNIH2R4LOVQPCZETW5YXKBM36FA7SJUDGNIH2R4LOVQPCZETW5YXKBM',
  arb:    'GCZETW5YXKBM36FA7SJUDGNIH2R4LOVQPCZETW5YXKBM36FA7SJUDGNI',
};
