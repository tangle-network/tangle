import {
	AliasDefinition,
	DefinitionRpc,
	DefinitionRpcSub,
	DefinitionsTypes,
	RegistryTypes,
} from "@polkadot/types/types";

export function jsonrpcFromDefs<
	Defs extends Record<string, { rpc?: Record<string, any> }>,
>(
	definitions: Defs,
	jsonrpc = {} as Record<
		string,
		Record<string, DefinitionRpc | DefinitionRpcSub>
	>,
) {
	Object.keys(definitions)
		.filter((key) => Object.keys(definitions[key]?.rpc ?? {}).length !== 0)
		.forEach((section): void => {
			jsonrpc[section] = {};

			Object.entries(definitions[section].rpc ?? {}).forEach(
				([method, def]): void => {
					const isSubscription = !!def.pubsub;

					jsonrpc[section][method] = {
						...def,
						isSubscription,
						jsonrpc: `${section}_${method}`,
						method,
						section,
					};
				},
			);
		});

	return jsonrpc;
}

export function typesAliasFromDefs<
	const Defs extends Record<string, { typesAlias?: AliasDefinition }>,
	ReturnType extends AliasDefinition,
>(definitions: Defs, initAlias = {} as ReturnType): ReturnType {
	return Object.values(definitions).reduce(
		(res: ReturnType, { typesAlias }): ReturnType => ({
			...typesAlias,
			...res,
		}),
		initAlias,
	);
}

export function typesFromDefs<
	Defs extends Record<string, { types: DefinitionsTypes }>,
>(definitions: Defs, initTypes = {} as RegistryTypes) {
	return Object.values(definitions).reduce(
		(res, { types }) => ({
			...res,
			...(types as RegistryTypes),
		}),
		initTypes,
	);
}
