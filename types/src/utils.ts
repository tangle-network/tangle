export function jsonrpcFromDefs(
  definitions: Record<string, { rpc?: Record<string, any> }>,
  jsonrpc: Record<string, Record<string, any>> = {}
): Record<string, Record<string, any>> {
  Object.keys(definitions)
    .filter(key => Object.keys(definitions[key]?.rpc ?? {}).length !== 0)
    .forEach((section): void => {
      jsonrpc[section] = {}
      Object.entries(definitions[section].rpc ?? {}).forEach(
        ([method, def]): void => {
          const isSubscription = !!def.pubsub

          jsonrpc[section][method] = {
            ...def,
            isSubscription,
            jsonrpc: `${section}_${method}`,
            method,
            section,
          }
        }
      )
    })

  return jsonrpc
}

export function typesAliasFromDefs(
  definitions: Record<string, Record<string, any>>,
  initAlias: Record<string, any> = {}
): Record<string, any> {
  return Object.values(definitions).reduce(
    (res: Record<string, any>, { typesAlias }): Record<string, any> => ({
      ...typesAlias,
      ...res,
    }),
    initAlias
  )
}

export function typesFromDefs(
  definitions: Record<string, { types: Record<string, any> }>,
  initTypes: Record<string, any> = {}
): Record<string, any> {
  return Object.values(definitions).reduce(
    (res: Record<string, any>, { types }): Record<string, any> => ({
      ...res,
      ...types,
    }),
    initTypes
  )
}
