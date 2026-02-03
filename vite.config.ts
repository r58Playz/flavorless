import type { UserConfig } from "vite";
import fs from "node:fs/promises";

export default {
	plugins: [
		{
			name: "cssom-monkeypatch",
			resolveId(source: string) {
				console.log("resolve");
				if (source === "virtual:rrweb-cssom") {
					return "\0" + source;
				}
				return null;
			},
			async load(source: string) {
				if (source === "\0virtual:rrweb-cssom") {
					let code = await fs.readFile("node_modules/rrweb-cssom/build/CSSOM.js");
					return `
						let exports = {};
						${code}
						export { CSSOM };
					`;
				}
				return null;
			},
		},
	]
} satisfies UserConfig;
