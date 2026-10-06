export default function CacheBadge({ hit }: { hit: boolean }) {
  return (
    <span
      className={`badge ${hit ? 'cache-hit' : 'cache-miss'}`}
      title={hit ? 'Served from the per-user cache' : 'Loaded from the database'}
    >
      cache: {hit ? 'HIT' : 'MISS'}
    </span>
  )
}
