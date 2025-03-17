import { bundle } from "bunchee";
import { existsSync, readFileSync, writeFileSync } from "fs";
import { resolve } from "path";

function updateInterfaceProperties() {
	const filePath = resolve("src/interfaces/types-lookup.ts");
	let content = readFileSync(filePath, "utf-8");

	// Replace properties in the IsmpRouterGetResponse interface
	content = content.replace(
		/interface IsmpRouterGetResponse extends Struct \{([^}]*?)get:([^;]*?);([^}]*?)values:([^;]*?);([^}]*?)\}/s,
		"interface IsmpRouterGetResponse extends Struct {$1getRequest:$2;$3getValues:$4;$5}",
	);

	writeFileSync(filePath, content);
	console.log(
		`✅ Updated IsmpRouterGetResponse interface properties inside ${filePath}`,
	);
}

function addMissingImport() {
	const filePath = resolve("src/interfaces/services/types.ts");
	const importContent = `import { TanglePrimitivesServicesFieldFieldType } from '@polkadot/types/lookup';`;

	// Check if the import exists
	if (!existsSync(filePath)) {
		console.warn(
			`⚠️ ${filePath} does not exist, ignoring add missing import for TanglePrimitivesServicesFieldFieldType`,
		);
		return;
	}

	const content = readFileSync(filePath, "utf-8");

	// Check if the import already exists
	if (content.includes(importContent)) {
		console.log(
			`✅ Import for TanglePrimitivesServicesFieldFieldType already exists inside ${filePath}`,
		);
		return;
	}

	// Add the content at line 3
	const lines = content.split("\n");
	lines.splice(3, 0, importContent);
	writeFileSync(filePath, lines.join("\n"));

	console.log(
		`✅ Added missing import for TanglePrimitivesServicesFieldFieldType inside ${filePath}`,
	);
}

async function main() {
	// Update interface properties before bundling
	updateInterfaceProperties();

	// Add missing import for TanglePrimitivesServicesFieldFieldType
	// Manually add until this issue is resolved: https://github.com/polkadot-js/api/issues/6117
	addMissingImport();

	await bundle("", {
		clean: true,
	});
}

main().then(
	() => {
		console.log("✅ Build successful");
		process.exit(0);
	},
	(error) => {
		console.error("❌ Build failed");
		console.error(error);
		process.exit(1);
	},
);
