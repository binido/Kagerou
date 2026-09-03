export const pluralize = (
  count: number,
  singular: string,
  plural = `${singular}s`,
) => `${count} ${count === 1 ? singular : plural}`

export const formatProfileCount = (count: number) => pluralize(count, 'profile')
export const formatSourceCount = (count: number) => pluralize(count, 'source')

export const formatLogCount = (count: number, query: string) =>
  query.trim()
    ? `${count} ${count === 1 ? 'matching entry' : 'matching entries'}`
    : `${count} entries`

export const formatSourceTimestamp = (value: string) => value
