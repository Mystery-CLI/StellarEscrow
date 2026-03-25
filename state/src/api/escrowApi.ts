import { createApi, fetchBaseQuery } from '@reduxjs/toolkit/query/react';
import { Trade, Event } from '../types';

export const escrowApi = createApi({
  reducerPath: 'escrowApi',
  baseQuery: fetchBaseQuery({ baseUrl: '/api' }),
  tagTypes: ['Trade', 'Event'],
  endpoints: (builder) => ({
    // Trades
    getTrades: builder.query<Trade[], { limit?: number; offset?: number }>({
      query: ({ limit = 50, offset = 0 }) => `/trades?limit=${limit}&offset=${offset}`,
      providesTags: ['Trade'],
    }),
    getTrade: builder.query<Trade, string>({
      query: (id) => `/trades/${id}`,
      providesTags: (result, error, id) => [{ type: 'Trade', id }],
    }),
    createTrade: builder.mutation<Trade, Partial<Trade>>({
      query: (trade) => ({
        url: '/trades',
        method: 'POST',
        body: trade,
      }),
      invalidatesTags: ['Trade'],
    }),
    updateTrade: builder.mutation<Trade, { id: string; data: Partial<Trade> }>({
      query: ({ id, data }) => ({
        url: `/trades/${id}`,
        method: 'PATCH',
        body: data,
      }),
      invalidatesTags: (result, error, { id }) => [{ type: 'Trade', id }],
    }),

    // Events
    getEvents: builder.query<Event[], { limit?: number; tradeId?: string }>({
      query: ({ limit = 100, tradeId }) => {
        const params = new URLSearchParams({ limit: limit.toString() });
        if (tradeId) params.append('tradeId', tradeId);
        return `/events?${params}`;
      },
      providesTags: ['Event'],
    }),
    getEventsByTrade: builder.query<Event[], string>({
      query: (tradeId) => `/events/trade/${tradeId}`,
      providesTags: (result, error, tradeId) => [{ type: 'Event', id: tradeId }],
    }),
  }),
});

export const {
  useGetTradesQuery,
  useGetTradeQuery,
  useCreateTradeMutation,
  useUpdateTradeMutation,
  useGetEventsQuery,
  useGetEventsByTradeQuery,
} = escrowApi;
