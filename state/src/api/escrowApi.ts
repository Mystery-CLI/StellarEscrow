import { createApi } from '@reduxjs/toolkit/query/react';
import { Trade, Event } from '../types';
import { createBaseQuery } from './baseQuery';

export const escrowApi = createApi({
  reducerPath: 'escrowApi',
  baseQuery: createBaseQuery(),
  tagTypes: ['Trade', 'Event'],
  endpoints: (builder) => ({
    // Trades
    getTrades: builder.query<Trade[], { limit?: number; offset?: number }>({
      query: ({ limit = 50, offset = 0 }: { limit?: number; offset?: number }) =>
        `/trades?limit=${limit}&offset=${offset}`,
      providesTags: ['Trade'],
    }),
    getTrade: builder.query<Trade, string>({
      query: (id: string) => `/trades/${id}`,
      providesTags: (_result, _error, id: string) => [{ type: 'Trade' as const, id }],
    }),
    createTrade: builder.mutation<Trade, Partial<Trade>>({
      query: (trade: Partial<Trade>) => ({
        url: '/trades',
        method: 'POST',
        body: trade,
      }),
      invalidatesTags: ['Trade'],
    }),
    updateTrade: builder.mutation<Trade, { id: string; data: Partial<Trade> }>({
      query: ({ id, data }: { id: string; data: Partial<Trade> }) => ({
        url: `/trades/${id}`,
        method: 'PATCH',
        body: data,
      }),
      invalidatesTags: (_result, _error, { id }: { id: string; data: Partial<Trade> }) => [
        { type: 'Trade' as const, id },
      ],
    }),

    // Events
    getEvents: builder.query<Event[], { limit?: number; tradeId?: string }>({
      query: ({ limit = 100, tradeId }: { limit?: number; tradeId?: string }) => {
        const params = new URLSearchParams({ limit: limit.toString() });
        if (tradeId) params.append('tradeId', tradeId);
        return `/events?${params}`;
      },
      providesTags: ['Event'],
    }),
    getEventsByTrade: builder.query<Event[], string>({
      query: (tradeId: string) => `/events/trade/${tradeId}`,
      providesTags: (_result, _error, tradeId: string) => [
        { type: 'Event' as const, id: tradeId },
      ],
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
