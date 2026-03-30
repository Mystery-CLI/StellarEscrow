import { createApi, fetchBaseQuery } from '@reduxjs/toolkit/query/react';
import type { Trade, Event } from '../types';

export const escrowApi = createApi({
  reducerPath: 'escrowApi',

  baseQuery: fetchBaseQuery({
    baseUrl: '/api',
  }),

  tagTypes: ['Trade', 'Event'],

  endpoints: (builder) => ({
    // =========================
    // TRADES
    // =========================

    getTrades: builder.query<Trade[], { limit?: number; offset?: number }>({
      query: ({ limit = 50, offset = 0 }) =>
        `/trades?limit=${limit}&offset=${offset}`,

      providesTags: (result) =>
        result
          ? [
              ...result.map((trade) => ({
                type: 'Trade' as const,
                id: trade.id,
              })),
              { type: 'Trade', id: 'LIST' },
            ]
          : [{ type: 'Trade', id: 'LIST' }],
    }),

    getTrade: builder.query<Trade, string>({
      query: (id: string) => `/trades/${id}`,

      providesTags: (result, error, id) => [
        { type: 'Trade', id },
      ],
    }),

    createTrade: builder.mutation<Trade, Partial<Trade>>({
      query: (trade: Partial<Trade>) => ({
        url: '/trades',
        method: 'POST',
        body: trade,
      }),

      invalidatesTags: [{ type: 'Trade', id: 'LIST' }],
    }),

    updateTrade: builder.mutation<
      Trade,
      { id: string; data: Partial<Trade> }
    >({
      query: ({ id, data }) => ({
        url: `/trades/${id}`,
        method: 'PATCH',
        body: data,
      }),

      invalidatesTags: (result, error, { id }) => [
        { type: 'Trade', id },
      ],
    }),

    // =========================
    // EVENTS
    // =========================

    getEvents: builder.query<
      Event[],
      { limit?: number; tradeId?: string }
    >({
      query: ({ limit = 100, tradeId }) => {
        const params = new URLSearchParams({
          limit: limit.toString(),
        });

        if (tradeId) {
          params.append('tradeId', tradeId);
        }

        return `/events?${params.toString()}`;
      },

      providesTags: (result) =>
        result
          ? [
              ...result.map((event) => ({
                type: 'Event' as const,
                id: event.id,
              })),
              { type: 'Event', id: 'LIST' },
            ]
          : [{ type: 'Event', id: 'LIST' }],
    }),

    getEventsByTrade: builder.query<Event[], string>({
      query: (tradeId: string) => `/events/trade/${tradeId}`,

      providesTags: (result, error, tradeId) => [
        { type: 'Event', id: tradeId },
      ],
    }),
  }),
});

// =========================
// EXPORT HOOKS
// =========================

export const {
  useGetTradesQuery,
  useGetTradeQuery,
  useCreateTradeMutation,
  useUpdateTradeMutation,
  useGetEventsQuery,
  useGetEventsByTradeQuery,
} = escrowApi;