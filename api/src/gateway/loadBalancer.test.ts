import { LoadBalancer } from './loadBalancer';
import { UpstreamConfig } from './types';

const upstreams: UpstreamConfig[] = [
  { name: 'indexer-a', url: 'http://indexer-a:3000', pathPrefix: '/events' },
  { name: 'indexer-b', url: 'http://indexer-b:3000', pathPrefix: '/events' },
  { name: 'search',    url: 'http://indexer:3000',   pathPrefix: '/search' },
];

describe('LoadBalancer', () => {
  it('returns null for unmatched path', () => {
    const lb = new LoadBalancer();
    expect(lb.select('/unknown', upstreams)).toBeNull();
  });

  it('returns the only matching upstream', () => {
    const lb = new LoadBalancer();
    const u = lb.select('/search/trades', upstreams);
    expect(u?.name).toBe('search');
  });

  it('round-robins across multiple upstreams for the same prefix', () => {
    const lb = new LoadBalancer();
    const first  = lb.select('/events', upstreams);
    const second = lb.select('/events', upstreams);
    const third  = lb.select('/events', upstreams);
    expect(first?.name).toBe('indexer-a');
    expect(second?.name).toBe('indexer-b');
    expect(third?.name).toBe('indexer-a');  // wraps around
  });

  it('prefers longer prefix match', () => {
    const lb = new LoadBalancer();
    const specific: UpstreamConfig[] = [
      { name: 'events-all',    url: 'http://a:3000', pathPrefix: '/events' },
      { name: 'events-replay', url: 'http://b:3000', pathPrefix: '/events/replay' },
    ];
    const u = lb.select('/events/replay', specific);
    expect(u?.name).toBe('events-replay');
  });
});
