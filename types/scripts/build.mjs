import { bundle } from "bunchee";
import { resolve } from "path";
import { readFile, writeFile } from "fs/promises";

async function updateInterfaceProperties() {
	const filePath = resolve("src/interfaces/types-lookup.ts");
	let content = await readFile(filePath, "utf-8");

	// Replace properties in the IsmpRouterGetResponse interface
	content = content.replace(
		/interface IsmpRouterGetResponse extends Struct \{([^}]*?)get:([^;]*?);([^}]*?)values:([^;]*?);([^}]*?)\}/s,
		"interface IsmpRouterGetResponse extends Struct {$1getRequest:$2;$3getValues:$4;$5}",
	);

	await writeFile(filePath, content);
	console.log(
		`✅ Updated IsmpRouterGetResponse interface properties inside ${filePath}`,
	);
}

async function main() {
	// Update interface properties before bundling
	await updateInterfaceProperties();

	// Then proceed with bundling
	await bundle(resolve("src/index.ts"));
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
